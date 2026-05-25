"""Server-side prepared-statement lifecycle regression.

Backs up the source-side change in `src/connection/impls.rs` that dropped the
explicit `DEALLOCATE PREPARE` after non-cached prepares and started relying on
tokio-postgres' `Drop for StatementInner` to send `Close('S', name) + Sync`
when the last `Arc<StatementInner>` clone is dropped.

If that wiring ever regresses (e.g. an outstanding clone keeps a Statement
alive past the consumer's `Result`), `pg_prepared_statements` will start
holding entries we never explicitly cleared, and the server-side resource
slowly grows. This test catches that by making sure a burst of non-cached
prepares lands at the same backend connection at zero prepared statements
after the calls return.
"""

import pytest
from psqlpy import ConnectionPool

pytestmark = pytest.mark.anyio

# The in-band approach `pg_prepared_statements` is per-session, so both
# tests issue the introspection query on the same `connection` object they
# populated — opening a second pool connection would land on a different
# backend with an empty per-session view. The introspection query itself
# runs with `prepared=False` so it doesn't perturb the cache it's
# measuring (no-parameter `prepared=False` uses tokio-postgres' unnamed
# prepared statement which is dropped immediately).


async def test_non_cached_prepare_does_not_leak_server_side(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
) -> None:
    """Non-cached prepares drop their Statement and send Close('S').

    Sequence:
      1. Open one pooled connection.
      2. Run `SELECT 1` with `prepared=False` 50 times in a row on the same
         connection.
      3. From the same connection, count rows in `pg_prepared_statements`.
         If the DEALLOCATE-removal kept its end of the bargain (Statement
         Drop → Close), the count is zero. If statements leak, the count
         grows roughly with the number of calls.
    """
    pool = ConnectionPool(
        username=postgres_user,
        password=postgres_password,
        host=postgres_host,
        port=postgres_port,
        db_name=postgres_dbname,
        max_db_pool_size=2,
    )
    try:
        connection = await pool.connection()

        for _ in range(50):
            await connection.execute("SELECT 1", prepared=False)

        # Same connection — `pg_prepared_statements` is per-session, so the
        # query has to ride the same backend. Use `prepared=False` here too
        # to avoid the introspection query itself populating the cache we're
        # measuring.
        leaked = await connection.execute(
            "SELECT count(*)::bigint AS n FROM pg_prepared_statements",
            prepared=False,
        )
        rows = leaked.result()
        assert len(rows) == 1, rows
        assert rows[0]["n"] == 0, (
            f"Expected 0 prepared statements after non-cached prepares, found "
            f"{rows[0]['n']}. This means tokio-postgres' Drop for "
            f"StatementInner did not send Close('S') — the DEALLOCATE-removal "
            f"in src/connection/impls.rs regressed."
        )
    finally:
        pool.close()


async def test_cached_prepare_retains_statements_while_held(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
) -> None:
    """`prepared=True` with parameters keeps the named statement alive.

    Dual of the previous test for the path that *does* go through deadpool's
    `prepare_cached`: queries that carry parameters route through the
    StatementBuilder, which prepares a named statement and the
    `deadpool_postgres::StatementCache` holds an `Arc<Statement>` clone. The
    Statement is therefore not dropped after each call, and the cached
    server-side prepared statement persists for the lifetime of the pooled
    connection. Re-executing the same query string should reuse the same
    cache entry, so `pg_prepared_statements` for that statement text shows
    exactly one row no matter how many times we execute.

    The no-parameter path (covered by `execute_no_params`) uses tokio's
    unnamed-prepared-statement shortcut and never populates the cache, so
    we deliberately use parameters here.
    """
    pool = ConnectionPool(
        username=postgres_user,
        password=postgres_password,
        host=postgres_host,
        port=postgres_port,
        db_name=postgres_dbname,
        max_db_pool_size=2,
    )
    try:
        connection = await pool.connection()

        # Parameterised → goes through StatementBuilder → prepare_cached.
        for _ in range(20):
            await connection.execute(
                "SELECT $1::int4 AS v",
                parameters=[7],
                prepared=True,
            )

        result = await connection.execute(
            "SELECT count(*)::bigint AS n FROM pg_prepared_statements "
            "WHERE statement LIKE 'SELECT $1::int4 AS v'",
            prepared=False,
        )
        rows = result.result()
        assert len(rows) == 1
        assert rows[0]["n"] == 1, (
            f"Expected exactly 1 cached prepared statement for the "
            f"parameterised query, found {rows[0]['n']}. Either the "
            f"deadpool StatementCache stopped reusing per-query entries or "
            f"the prepared=True path stopped going through prepare_cached."
        )
    finally:
        pool.close()
