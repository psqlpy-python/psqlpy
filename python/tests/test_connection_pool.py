import pytest
from psqlpy import (
    Connection,
    ConnectionPool,
    ConnRecyclingMethod,
    LoadBalanceHosts,
    TargetSessionAttrs,
    connect,
)
from psqlpy.exceptions import (
    ConnectionPoolConfigurationError,
    RustPSQLDriverPyBaseError,
)

pytestmark = pytest.mark.anyio


async def test_connect_func() -> None:
    """Test that connect function makes new connection pool."""
    pg_pool = connect(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_pool_dsn_startup() -> None:
    """Test that connection pool can startup with dsn."""
    pg_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


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

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_build_pool_failure() -> None:
    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(
            dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
            connect_timeout_nanosec=12,
        )
    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(
            dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
            connect_timeout_nanosec=12,
        )
    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(
            dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
            keepalives_idle_nanosec=12,
        )
    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(
            dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
            keepalives_interval_nanosec=12,
        )


@pytest.mark.parametrize(
    "target_session_attrs",
    [
        TargetSessionAttrs.Any,
        TargetSessionAttrs.ReadWrite,
        TargetSessionAttrs.ReadOnly,
    ],
)
async def test_pool_target_session_attrs(
    target_session_attrs: TargetSessionAttrs,
) -> None:
    pg_pool = ConnectionPool(
        db_name="psqlpy_test",
        host="localhost",
        username="postgres",
        password="postgres",  # noqa: S106
        target_session_attrs=target_session_attrs,
    )

    if target_session_attrs == TargetSessionAttrs.ReadOnly:
        with pytest.raises(expected_exception=RustPSQLDriverPyBaseError):
            await pg_pool.connection()
    else:
        conn = await pg_pool.connection()
        await conn.execute("SELECT 1")


@pytest.mark.parametrize(
    "load_balance_hosts",
    [
        LoadBalanceHosts.Disable,
        LoadBalanceHosts.Random,
    ],
)
async def test_pool_load_balance_hosts(
    load_balance_hosts: LoadBalanceHosts,
) -> None:
    pg_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
        load_balance_hosts=load_balance_hosts,
    )

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_close_connection_pool() -> None:
    """Test that `close` method closes connection pool."""
    pg_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")

    pg_pool.close()

    with pytest.raises(expected_exception=RustPSQLDriverPyBaseError):
        await pg_pool.connection()


async def test_connection_pool_as_context_manager() -> None:
    """Test connection pool as context manager."""
    with ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    ) as pg_pool:
        conn = await pg_pool.connection()
        res = await conn.execute("SELECT 1")
        assert res.result()

    with pytest.raises(expected_exception=RustPSQLDriverPyBaseError):
        await pg_pool.connection()
