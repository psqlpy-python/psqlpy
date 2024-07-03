import psqlpy_piccolo
import psqlpy_piccolo.engine
from piccolo.engine.postgres import PostgresEngine

from psqlpy_stress.helpers import parse_postgres_url
from psqlpy_stress.settings import settings


ASYNCPG_PICCOLO_ENGINE = PostgresEngine(
    config=parse_postgres_url(settings.database_url),
)
PSQLPY_PICCOLO_ENGINE = psqlpy_piccolo.engine.PSQLPyEngine(
    config=parse_postgres_url(settings.database_url),
)


async def start_piccolo_pools() -> None:
    await ASYNCPG_PICCOLO_ENGINE.start_connection_pool(
        max_size=settings.max_pool_size,
        min_size=1,
    )
    await PSQLPY_PICCOLO_ENGINE.start_connection_pool(
        max_size=settings.max_pool_size,
    )

    print("Piccolo pools started.")


async def close_piccolo_pools() -> None:
    for engine in [
        ASYNCPG_PICCOLO_ENGINE,
        PSQLPY_PICCOLO_ENGINE,
    ]:
        await engine.close_connection_pool()

    print("Piccolo pools closed.")
