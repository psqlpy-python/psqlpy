import pytest

from psqlpy import Connection, ConnectionPool, ConnRecyclingMethod, QueryResult, connect
from psqlpy.exceptions import RustPSQLDriverPyBaseError

pytestmark = pytest.mark.anyio


async def test_connect_func() -> None:
    """Test that connect function makes new connection pool."""
    pg_pool = connect(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    await pg_pool.execute("SELECT 1")


async def test_pool_dsn_startup() -> None:
    """Test that connection pool can startup with dsn."""
    pg_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    await pg_pool.execute("SELECT 1")


async def test_pool_execute(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test that ConnectionPool can execute queries."""
    select_result = await psql_pool.execute(
        f"SELECT * FROM {table_name}",
    )

    assert type(select_result) == QueryResult

    inner_result = select_result.result()
    assert isinstance(inner_result, list)
    assert len(inner_result) == number_database_records


async def test_pool_connection(
    psql_pool: ConnectionPool,
) -> None:
    """Test that ConnectionPool can return single connection from the pool."""
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
    pg_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
        conn_recycling_method=conn_recycling_method,
    )

    await pg_pool.execute("SELECT 1")


async def test_close_connection_pool() -> None:
    """Test that `close` method closes connection pool."""
    pg_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    await pg_pool.execute("SELECT 1")

    pg_pool.close()

    with pytest.raises(expected_exception=RustPSQLDriverPyBaseError):
        await pg_pool.execute("SELECT 1")
