import typing
import uuid
from datetime import date, datetime, timezone
from decimal import Decimal

import pytest
from psqlpy import ConnectionPool
from psqlpy.exceptions import PyToRustValueMappingError

pytestmark = pytest.mark.anyio


async def _setup_target_table(psql_pool: ConnectionPool, name: str) -> None:
    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP TABLE IF EXISTS {name}")
        await connection.execute(
            f"""
            CREATE TABLE {name} (
                id INTEGER,
                label TEXT,
                weight DOUBLE PRECISION,
                created_at TIMESTAMPTZ
            )
            """,
        )


async def _drop_target_table(psql_pool: ConnectionPool, name: str) -> None:
    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP TABLE IF EXISTS {name}")


async def test_copy_records_to_table_on_connection(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_conn"
    await _setup_target_table(psql_pool, target)
    try:
        records = [
            (1, "alpha", 1.5, datetime(2026, 1, 1, tzinfo=timezone.utc)),
            (2, "beta", 2.25, datetime(2026, 1, 2, tzinfo=timezone.utc)),
            (3, "gamma", None, datetime(2026, 1, 3, tzinfo=timezone.utc)),
        ]

        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
            )

        assert inserted == len(records)

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label, weight FROM {target} ORDER BY id",
            )
        rows = result.result()
        assert [(r["id"], r["label"], r["weight"]) for r in rows] == [
            (1, "alpha", 1.5),
            (2, "beta", 2.25),
            (3, "gamma", None),
        ]
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_with_columns_subset(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_subset"
    await _setup_target_table(psql_pool, target)
    try:
        records = [(10, "only-label"), (11, "another")]

        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
                columns=["id", "label"],
            )

        assert inserted == len(records)

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label, weight, created_at FROM {target} ORDER BY id",
            )
        rows = result.result()
        assert [(r["id"], r["label"]) for r in rows] == [
            (10, "only-label"),
            (11, "another"),
        ]
        # Untouched columns must remain NULL
        assert all(r["weight"] is None and r["created_at"] is None for r in rows)
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_in_transaction(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_tx"
    await _setup_target_table(psql_pool, target)
    try:
        records = [(100, "tx-row", 0.0, datetime(2026, 5, 1, tzinfo=timezone.utc))]

        async with (
            psql_pool.acquire() as connection,
            connection.transaction() as tx,
        ):
            inserted = await tx.copy_records_to_table(
                table_name=target,
                records=records,
            )

        assert inserted == 1

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT COUNT(*) AS c FROM {target}",
            )
        assert result.result()[0]["c"] == 1
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_rejects_record_arity_mismatch(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_mismatch"
    await _setup_target_table(psql_pool, target)
    try:
        records = [(1, "missing-rest")]  # table has 4 columns

        async with psql_pool.acquire() as connection:
            with pytest.raises(PyToRustValueMappingError):
                await connection.copy_records_to_table(
                    table_name=target,
                    records=records,
                )
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_uses_schema_qualifier(
    psql_pool: ConnectionPool,
) -> None:
    schema: typing.Final = "copy_records_schema"
    target: typing.Final = "tbl"

    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP SCHEMA IF EXISTS {schema} CASCADE")
        await connection.execute(f"CREATE SCHEMA {schema}")
        await connection.execute(
            f"CREATE TABLE {schema}.{target} (id INTEGER, label TEXT)",
        )

    try:
        records = [(1, "schema-a"), (2, "schema-b")]
        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
                schema_name=schema,
            )

        assert inserted == len(records)

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label FROM {schema}.{target} ORDER BY id",
            )
        assert [(r["id"], r["label"]) for r in result.result()] == records
    finally:
        async with psql_pool.acquire() as connection:
            await connection.execute(f"DROP SCHEMA IF EXISTS {schema} CASCADE")


async def test_copy_records_heterogeneous_types(
    psql_pool: ConnectionPool,
) -> None:
    """Characterization test: covers int, float, text, bytea, UUID, numeric,
    date, timestamp, NULL, and array column types (AC-3.4).
    """
    target: typing.Final = "copy_records_hetero"

    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP TABLE IF EXISTS {target}")
        await connection.execute(
            f"""
            CREATE TABLE {target} (
                col_int      INTEGER,
                col_float    DOUBLE PRECISION,
                col_text     TEXT,
                col_bytea    BYTEA,
                col_uuid     UUID,
                col_numeric  NUMERIC,
                col_date     DATE,
                col_ts       TIMESTAMPTZ,
                col_null     TEXT,
                col_arr      INTEGER[]
            )
            """,
        )

    try:
        sample_uuid = uuid.uuid4()
        records = [
            (
                42,
                3.14,
                "hello",
                b"\x00\x01\x02",
                sample_uuid,
                Decimal("12345.6789"),
                date(2024, 6, 1),
                datetime(2024, 6, 1, 12, 0, 0, tzinfo=timezone.utc),
                None,
                [1, 2, 3],
            ),
        ]

        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
            )

        assert inserted == 1

        async with psql_pool.acquire() as connection:
            result = await connection.execute(f"SELECT * FROM {target}")
        row = result.result()[0]
        assert row["col_int"] == 42
        assert abs(row["col_float"] - 3.14) < 1e-9
        assert row["col_text"] == "hello"
        assert bytes(row["col_bytea"]) == b"\x00\x01\x02"
        assert row["col_uuid"] == str(sample_uuid)
        assert row["col_numeric"] == Decimal("12345.6789")
        assert row["col_date"] == date(2024, 6, 1)
        assert row["col_null"] is None
        assert row["col_arr"] == [1, 2, 3]
    finally:
        async with psql_pool.acquire() as connection:
            await connection.execute(f"DROP TABLE IF EXISTS {target}")


async def test_copy_records_introspection_cache(
    psql_pool: ConnectionPool,
) -> None:
    """Second call to copy_records_to_table against the same table should not
    issue a new column-type introspection PREPARE (AC-4.3).
    """
    target: typing.Final = "copy_records_cache_test"
    records = [(1, "first"), (2, "second")]

    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP TABLE IF EXISTS {target}")
        await connection.execute(
            f"CREATE TABLE {target} (id INTEGER, label TEXT)",
        )

    # Snapshot introspection query count before — use pg_stat_statements if available.
    introspect_pattern = f"%{target}%WHERE false%"
    pre_calls: int | None = None
    try:
        async with psql_pool.acquire() as connection:
            res = await connection.execute(
                "SELECT COALESCE(SUM(calls), 0) AS n FROM pg_stat_statements "
                "WHERE query ILIKE $1",
                parameters=[introspect_pattern],
            )
        pre_calls = res.result()[0]["n"]
    except Exception:  # noqa: BLE001, S110
        pass  # pg_stat_statements not available — skip count check

    try:
        async with psql_pool.acquire() as connection:
            # First call — populates the cache.
            await connection.copy_records_to_table(
                table_name=target,
                records=records[:1],
            )
            # Second call on the same connection — must hit the type cache.
            await connection.copy_records_to_table(
                table_name=target,
                records=records[1:],
            )

        # Verify both rows were written correctly.
        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label FROM {target} ORDER BY id",
            )
        rows = [(r["id"], r["label"]) for r in result.result()]
        assert rows == [(1, "first"), (2, "second")]

        # Verify only one introspection query was issued (cache hit on second call).
        if pre_calls is not None:
            async with psql_pool.acquire() as connection:
                res = await connection.execute(
                    "SELECT COALESCE(SUM(calls), 0) AS n FROM pg_stat_statements "
                    "WHERE query ILIKE $1",
                    parameters=[introspect_pattern],
                )
            post_calls = res.result()[0]["n"]
            # At most one introspection PREPARE should be issued (cache hit on call 2).
            assert post_calls - pre_calls <= 1, (
                f"Expected at most 1 introspection call, got {post_calls - pre_calls}"
            )
    finally:
        async with psql_pool.acquire() as connection:
            await connection.execute(f"DROP TABLE IF EXISTS {target}")
