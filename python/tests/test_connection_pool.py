import pytest

from psqlpy import Connection, ConnRecyclingMethod, PSQLPool, QueryResult

pytestmark = pytest.mark.anyio


async def test_pool_dsn_startup() -> None:
    """Test that connection pool can startup with dsn."""
    pg_pool = PSQLPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )
    await pg_pool.startup()

    await pg_pool.execute("SELECT 1")


async def test_pool_execute(
    psql_pool: PSQLPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test that PSQLPool can execute queries."""
    select_result = await psql_pool.execute(
        f"SELECT * FROM {table_name}",
    )

    assert type(select_result) == QueryResult

    inner_result = select_result.result()
    assert isinstance(inner_result, list)
    assert len(inner_result) == number_database_records


async def test_pool_connection(
    psql_pool: PSQLPool,
) -> None:
    """Test that PSQLPool can return single connection from the pool."""
    connection = await psql_pool.connection()
    assert isinstance(connection, Connection)


@pytest.mark.parametrize(
    "conn_recycling_method",
    [
        ConnRecyclingMethod.Fast,
        ConnRecyclingMethod.Verified,
        ConnRecyclingMethod.Clean,
    ],
)
async def test_pool_conn_recycling_method(
    conn_recycling_method: ConnRecyclingMethod,
) -> None:
    pg_pool = PSQLPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
        conn_recycling_method=conn_recycling_method,
    )

    await pg_pool.startup()

    await pg_pool.execute("SELECT 1")
