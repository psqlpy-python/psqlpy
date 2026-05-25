"""End-to-end check that `SslMode.Allow` actually retries over TLS.

Pre-requisite: a `pg_hba.conf` that contains only `hostssl` entries (no
plain `host` entries) for the TCP rows. With that config:

  - A plaintext connection attempt is rejected by the server with SQLSTATE
    `28000` (`INVALID_AUTHORIZATION_SPECIFICATION`) and the message
    "no encryption". This is what `PsqlpyManager::is_ssl_required_rejection`
    keys off.
  - A TLS connection succeeds.

Three assertions:
  (a) `ssl_mode=Allow` + a CA file succeeds AND postgres logs show a
      preceding plaintext rejection from the same test window — proves the
      retry actually fired, distinguishing it from a degenerate "Allow ==
      Require" implementation that would skip plaintext entirely.
  (b) `ssl_mode=Disable` fails with a `BaseConnectionPoolError` — there is
      no fallback, so the rejection bubbles up as a connection-pool
      backend failure. (psqlpy wraps tokio-postgres errors so the
      SQLSTATE/"no encryption" text doesn't survive to Python; the
      docker-log probe below carries the SQLSTATE signal when available.)
  (c) `ssl_mode=Require` + a CA file succeeds — control case, never goes
      through the plaintext side.

These tests are skipped automatically unless `PSQLPY_HOSTSSL_PORT` is set,
because the hostssl-only postgres is bespoke infra, not the regular
`POSTGRES_PORT` server used by the rest of the suite. If
`PSQLPY_HOSTSSL_DOCKER_CONTAINER` is also set, the Allow / Disable cases
additionally assert against `docker logs` from that container — that's
how we observe the retry actually firing rather than inferring it from a
green query alone.
"""

import os
import subprocess
import time

import pytest
from psqlpy import ConnectionPool, SslMode
from psqlpy.exceptions import BaseConnectionPoolError

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


def _docker_logs_since(container: str, since_epoch: float) -> str | None:
    """Return docker logs from `container` newer than `since_epoch`.

    Returns None when docker isn't available, the container doesn't exist,
    or the lookup fails for any other reason — callers treat the absence
    of logs as "skip the assertion" rather than as a test failure, because
    the assertion is opt-in (only meaningful with the matching docker
    infra). When the lookup succeeds, the return value is the combined
    stdout+stderr of `docker logs --since=<ts>` decoded as utf-8.
    """
    try:
        since_iso = time.strftime(
            "%Y-%m-%dT%H:%M:%S",
            time.gmtime(since_epoch),
        )
        proc = subprocess.run(  # noqa: S603 — container name is from env
            ["docker", "logs", "--since", since_iso, container],  # noqa: S607
            capture_output=True,
            check=False,
            timeout=5,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    return proc.stdout.decode("utf-8", errors="replace") + proc.stderr.decode(
        "utf-8",
        errors="replace",
    )


async def test_allow_retries_over_tls_when_hostssl_only(
    hostssl_host: str,
    hostssl_port: int,
    hostssl_user: str,
    hostssl_password: str,
    hostssl_dbname: str,
    hostssl_cert_file: str,
) -> None:
    """`SslMode.Allow` succeeds AND the retry path actually fires.

    Without observing the retry directly, this test would also pass for a
    degenerate implementation that mapped `Allow → Require` and skipped
    the plaintext attempt entirely. To distinguish those two
    implementations, we open the connection inside a window we can read
    back from the postgres server log: if a plaintext attempt was made,
    the server emitted a FATAL with SQLSTATE `28000` + the literal
    "no encryption" string within that window. The log probe is opt-in
    (`PSQLPY_HOSTSSL_DOCKER_CONTAINER`); when not provided we still
    assert the green query result, but flag the weakness in skip text.
    """
    container = os.environ.get("PSQLPY_HOSTSSL_DOCKER_CONTAINER")
    window_start = time.time()
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

    if container is None:
        pytest.skip(
            "set PSQLPY_HOSTSSL_DOCKER_CONTAINER to also assert the retry "
            "actually fired (the green query alone doesn't distinguish "
            "true Allow retry from a degenerate Allow==Require mapping)",
        )
        msg = "unreachable: pytest.skip raises"
        raise RuntimeError(msg)

    logs = _docker_logs_since(container, window_start)
    assert logs is not None, (
        f"docker logs for container {container!r} were unreadable; cannot "
        "verify that the plaintext attempt actually happened."
    )
    assert "no encryption" in logs, (
        "Expected postgres server log to contain a 'no encryption' "
        "rejection in the test window — that's the signal the Allow path "
        "made the plaintext attempt first. Its absence means the "
        "implementation may have skipped plaintext and gone straight to "
        f"TLS (i.e. behaved as Require). Full log snippet:\n{logs[-2000:]}"
    )


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
    container = os.environ.get("PSQLPY_HOSTSSL_DOCKER_CONTAINER")
    window_start = time.time()
    pool = ConnectionPool(
        username=hostssl_user,
        password=hostssl_password,
        host=hostssl_host,
        port=hostssl_port,
        db_name=hostssl_dbname,
        ssl_mode=SslMode.Disable,
    )
    try:
        with pytest.raises(BaseConnectionPoolError):
            await _connect_and_select_one(pool)
    finally:
        pool.close()

    # psqlpy wraps tokio-postgres errors as a flat string with no SQLSTATE
    # attribute exposed to Python; "db error" is all that survives the
    # PoolError → Error → Display chain. So we cannot pin the SQLSTATE
    # in-band — the SQLSTATE check rides on the postgres server log when
    # docker access is available. Without it we still get the
    # `BaseConnectionPoolError`-narrowed assertion above, which proves the
    # connection failed (not a different unrelated exception).
    if container is None:
        return
    logs = _docker_logs_since(container, window_start)
    if logs is None:
        return
    assert "no encryption" in logs, (
        "Expected the postgres server log to contain a 'no encryption' "
        "rejection during the Disable-against-hostssl-only attempt. Its "
        "absence means the failure mode wasn't the SSL-required diagnostic "
        f"this test is meant to cover. Recent log:\n{logs[-2000:]}"
    )


async def _connect_and_select_one(pool: ConnectionPool) -> None:
    """Tiny helper so the `pytest.raises` body stays a single statement.

    Either `pool.connection()` or the subsequent `execute` may raise
    depending on whether the rejection lands during the TCP handshake or
    during the first round-trip; the test cares only that the
    `BaseConnectionPoolError`-narrowed failure surfaces somewhere along
    this two-step call.
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
