import pytest
from psqlpy import (
    Connection,
    ConnectionPool,
    ConnRecyclingMethod,
    LoadBalanceHosts,
    TargetSessionAttrs,
    connect_pool,
)
from psqlpy.exceptions import (
    ConnectionPoolConfigurationError,
    InterfaceError,
)

pytestmark = pytest.mark.anyio


async def test_connect_func(dsn: str) -> None:
    """Test that connect function makes new connection pool."""
    pg_pool = connect_pool(dsn=dsn)

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_pool_dsn_startup(dsn: str) -> None:
    """Test that connection pool can startup with dsn."""
    pg_pool = ConnectionPool(dsn=dsn)

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_pool_connection(psql_pool: ConnectionPool) -> None:
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
    dsn: str,
) -> None:
    pg_pool = ConnectionPool(
        dsn=dsn,
        conn_recycling_method=conn_recycling_method,
    )

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_build_pool_failure(dsn: str) -> None:
    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(dsn=dsn, connect_timeout_nanosec=12)

    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(dsn=dsn, connect_timeout_nanosec=12)

    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(dsn=dsn, keepalives_idle_nanosec=12)

    with pytest.raises(expected_exception=ConnectionPoolConfigurationError):
        ConnectionPool(dsn=dsn, keepalives_interval_nanosec=12)


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
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
) -> None:
    pg_pool = ConnectionPool(
        db_name=postgres_dbname,
        host=postgres_host,
        port=postgres_port,
        username=postgres_user,
        password=postgres_password,
        target_session_attrs=target_session_attrs,
    )

    if target_session_attrs == TargetSessionAttrs.ReadOnly:
        with pytest.raises(expected_exception=InterfaceError):
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
    dsn: str,
) -> None:
    pg_pool = ConnectionPool(dsn=dsn, load_balance_hosts=load_balance_hosts)

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


async def test_close_connection_pool(dsn: str) -> None:
    """Test that `close` method closes connection pool."""
    pg_pool = ConnectionPool(dsn=dsn)

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")

    pg_pool.close()

    with pytest.raises(expected_exception=InterfaceError):
        await pg_pool.connection()


async def test_connection_pool_as_context_manager(dsn: str) -> None:
    """Test connection pool as context manager."""
    with ConnectionPool(dsn=dsn) as pg_pool:
        conn = await pg_pool.connection()
        res = await conn.execute("SELECT 1")
        assert res.result()

    with pytest.raises(expected_exception=InterfaceError):
        await pg_pool.connection()
