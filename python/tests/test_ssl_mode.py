import pytest

from psqlpy import ConnectionPool, SslMode
from psqlpy._internal import ConnectionPoolBuilder

pytestmark = pytest.mark.anyio


@pytest.mark.parametrize(
    "ssl_mode",
    (
        SslMode.Disable,
        SslMode.Allow,
        SslMode.Prefer,
        SslMode.Require,
        SslMode.VerifyCa,
        SslMode.VerifyFull,
    ),
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

    await pg_pool.execute("SELECT 1")


@pytest.mark.parametrize(
    "ssl_mode",
    (
        SslMode.Disable,
        SslMode.Allow,
        SslMode.Prefer,
        SslMode.Require,
        SslMode.VerifyCa,
        SslMode.VerifyFull,
    ),
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

    await pool.execute("SELECT 1")
