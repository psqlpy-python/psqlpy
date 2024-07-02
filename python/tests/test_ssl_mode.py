import pytest

from psqlpy import ConnectionPool, SslMode

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
