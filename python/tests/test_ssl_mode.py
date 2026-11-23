import pytest
from psqlpy import ConnectionPool, SslMode
from psqlpy._internal import ConnectionPoolBuilder

from tests.helpers import abandon, eventually, is_not_running, row_exists, row_locked

pytestmark = pytest.mark.anyio


@pytest.mark.parametrize(
    "ssl_mode",
    [
        SslMode.Disable,
        SslMode.Allow,
        SslMode.Prefer,
        SslMode.Require,
        SslMode.VerifyCa,
        SslMode.VerifyFull,
    ],
)
async def test_ssl_mode_require(
    ssl_mode: SslMode,
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
    ssl_cert_file: str,
) -> None:
    pg_pool = ConnectionPool(
        username=postgres_user,
        password=postgres_password,
        host=postgres_host,
        port=postgres_port,
        db_name=postgres_dbname,
        ssl_mode=ssl_mode,
        ca_file=ssl_cert_file,
    )

    conn = await pg_pool.connection()
    await conn.execute("SELECT 1")


@pytest.mark.parametrize(
    "ssl_mode",
    [
        SslMode.Disable,
        SslMode.Allow,
        SslMode.Prefer,
        SslMode.Require,
        SslMode.VerifyCa,
        SslMode.VerifyFull,
    ],
)
async def test_ssl_mode_require_pool_builder(
    ssl_mode: SslMode,
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
    ssl_cert_file: str,
) -> None:
    builder = (
        ConnectionPoolBuilder()
        .max_pool_size(10)
        .host(postgres_host)
        .port(postgres_port)
        .user(postgres_user)
        .password(postgres_password)
        .dbname(postgres_dbname)
        .ssl_mode(ssl_mode)
        .ca_file(ssl_cert_file)
    )

    pool = builder.build()

    connection = await pool.connection()
    await connection.execute("SELECT 1")


async def test_ssl_mode_require_without_ca_file(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
) -> None:
    builder = (
        ConnectionPoolBuilder()
        .max_pool_size(10)
        .host(postgres_host)
        .port(postgres_port)
        .user(postgres_user)
        .password(postgres_password)
        .dbname(postgres_dbname)
        .ssl_mode(SslMode.Require)
    )
    pool = builder.build()

    connection = await pool.connection()
    await connection.execute("SELECT 1")


async def test_cancelled_query_over_tls_is_cancelled_on_server(
    psql_pool: ConnectionPool,
    psql_pool_with_cert_file: ConnectionPool,
    table_name: str,
) -> None:
    """The cancel request negotiates TLS the way the pool's connections do."""
    query = f"DELETE FROM {table_name} WHERE id = 1"

    async def delete() -> None:
        async with psql_pool_with_cert_file.acquire() as connection:
            await connection.execute(query)

    async with row_locked(psql_pool, table_name):
        await abandon(psql_pool, query, delete())
        assert await eventually(lambda: is_not_running(psql_pool, query))

    assert await row_exists(psql_pool, table_name, 1)
