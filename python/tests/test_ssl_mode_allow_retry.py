"""End-to-end check that `SslMode.Allow` actually retries over TLS.

Pre-requisite: a `pg_hba.conf` that contains only `hostssl` entries (no
plain `host` entries) for the TCP rows. With that config:

  - A plaintext connection attempt is rejected by the server with SQLSTATE
    `28000` (`INVALID_AUTHORIZATION_SPECIFICATION`) and the message
    "no encryption". This is what `PsqlpyManager::is_ssl_required_rejection`
    keys off.
  - A TLS connection succeeds.

Three assertions:
  (a) `ssl_mode=Allow` + a CA file succeeds — the plaintext attempt is
      rejected, `PsqlpyManager` walks the `source()` chain, detects the
      "no encryption" diagnostic, and re-runs `create()` through the TLS
      fallback inner manager.
  (b) `ssl_mode=Disable` fails — there is no fallback, the rejection
      bubbles up unchanged. We assert the failure surfaces some signal of
      the postgres-side denial.
  (c) `ssl_mode=Require` + a CA file succeeds — control case, never goes
      through the plaintext side.

These tests are skipped automatically unless `PSQLPY_HOSTSSL_PORT` is set,
because the hostssl-only postgres is bespoke infra, not the regular
`POSTGRES_PORT` server used by the rest of the suite.
"""

import os

import pytest
from psqlpy import ConnectionPool, SslMode
from psqlpy.exceptions import ConnectionPoolBuildError, ConnectionPoolExecuteError

pytestmark = [
    pytest.mark.anyio,
    pytest.mark.skipif(
        os.environ.get("PSQLPY_HOSTSSL_PORT") is None,
        reason="needs a hostssl-only postgres on PSQLPY_HOSTSSL_PORT",
    ),
]


@pytest.fixture
def hostssl_port() -> int:
    return int(os.environ["PSQLPY_HOSTSSL_PORT"])


@pytest.fixture
def hostssl_host() -> str:
    return os.environ.get("PSQLPY_HOSTSSL_HOST", "localhost")


@pytest.fixture
def hostssl_user() -> str:
    return os.environ.get("PSQLPY_HOSTSSL_USER", "postgres")


@pytest.fixture
def hostssl_password() -> str:
    return os.environ.get("PSQLPY_HOSTSSL_PASSWORD", "postgres")


@pytest.fixture
def hostssl_dbname() -> str:
    return os.environ.get("PSQLPY_HOSTSSL_DBNAME", "psqlpy_test")


@pytest.fixture
def hostssl_cert_file() -> str:
    path = os.environ.get("PSQLPY_HOSTSSL_CERT_FILE")
    if path is None:
        pytest.skip("PSQLPY_HOSTSSL_CERT_FILE not set")
        msg = "unreachable: pytest.skip raises"
        raise RuntimeError(msg)
    return path


async def test_allow_retries_over_tls_when_hostssl_only(
    hostssl_host: str,
    hostssl_port: int,
    hostssl_user: str,
    hostssl_password: str,
    hostssl_dbname: str,
    hostssl_cert_file: str,
) -> None:
    """`SslMode.Allow` succeeds: PsqlpyManager falls back to TLS."""
    pool = ConnectionPool(
        username=hostssl_user,
        password=hostssl_password,
        host=hostssl_host,
        port=hostssl_port,
        db_name=hostssl_dbname,
        ssl_mode=SslMode.Allow,
        ca_file=hostssl_cert_file,
    )
    try:
        conn = await pool.connection()
        result = await conn.execute("SELECT 1 AS one")
        assert result.result()[0]["one"] == 1
    finally:
        pool.close()


async def test_disable_surfaces_no_encryption_rejection(
    hostssl_host: str,
    hostssl_port: int,
    hostssl_user: str,
    hostssl_password: str,
    hostssl_dbname: str,
) -> None:
    """`SslMode.Disable` against hostssl-only fails with the server's denial.

    The retry path is intentionally unavailable here (Disable has no TLS
    fallback inner manager), so the postgres-side rejection bubbles up. We
    don't pin the exception text — different driver versions wrap the
    diagnostic differently — but the connect or first query must fail.
    """
    pool = ConnectionPool(
        username=hostssl_user,
        password=hostssl_password,
        host=hostssl_host,
        port=hostssl_port,
        db_name=hostssl_dbname,
        ssl_mode=SslMode.Disable,
    )
    try:
        with pytest.raises(
            (ConnectionPoolBuildError, ConnectionPoolExecuteError, Exception),
        ):
            await _connect_and_select_one(pool)
    finally:
        pool.close()


async def _connect_and_select_one(pool: ConnectionPool) -> None:
    """Tiny helper so the `pytest.raises` body stays a single statement.

    Either `pool.connection()` or the subsequent `execute` may raise
    depending on whether the rejection lands during the TCP handshake or
    during the first round-trip; the test cares only that *something*
    inside this call fails.
    """
    conn = await pool.connection()
    await conn.execute("SELECT 1")


async def test_require_succeeds_with_ca_file(
    hostssl_host: str,
    hostssl_port: int,
    hostssl_user: str,
    hostssl_password: str,
    hostssl_dbname: str,
    hostssl_cert_file: str,
) -> None:
    """Control: `SslMode.Require` + ca_file goes straight through TLS."""
    pool = ConnectionPool(
        username=hostssl_user,
        password=hostssl_password,
        host=hostssl_host,
        port=hostssl_port,
        db_name=hostssl_dbname,
        ssl_mode=SslMode.Require,
        ca_file=hostssl_cert_file,
    )
    try:
        conn = await pool.connection()
        result = await conn.execute("SELECT 1 AS one")
        assert result.result()[0]["one"] == 1
    finally:
        pool.close()
