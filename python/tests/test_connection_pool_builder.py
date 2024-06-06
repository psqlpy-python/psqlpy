import pytest

from psqlpy import (
    ConnectionPoolBuilder,
    ConnRecyclingMethod,
    LoadBalanceHosts,
    SslMode,
    TargetSessionAttrs,
)

pytestmark = pytest.mark.anyio


async def test_connection_pool_builder(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
    table_name: str,
) -> None:
    """Test connection pool builder functionality."""
    builder = (
        ConnectionPoolBuilder()
        .max_pool_size(10)
        .host(postgres_host)
        .port(postgres_port)
        .user(postgres_user)
        .password(postgres_password)
        .dbname(postgres_dbname)
        .conn_recycling_method(
            ConnRecyclingMethod.Verified,
        )
        .options("")
        .application_name("testing")
        .ssl_mode(SslMode.Disable)
        .connect_timeout(10)
        .tcp_user_timeout(100)
        .target_session_attrs(
            TargetSessionAttrs.ReadWrite,
        )
        .load_balance_hosts(
            LoadBalanceHosts.Disable,
        )
        .keepalives(True)
        .keepalives_idle(10)
        .keepalives_interval(10)
        .keepalives_retries(10)
    )

    pool = builder.build()

    results = await pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )

    assert results.result()
