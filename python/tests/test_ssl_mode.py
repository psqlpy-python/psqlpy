import pytest

from psqlpy import ConnectionPool

pytestmark = pytest.mark.anyio


async def test_ssl_mode_require(psql_pool_with_cert_file: ConnectionPool) -> None:
    await psql_pool_with_cert_file.execute("SELECT 1")
