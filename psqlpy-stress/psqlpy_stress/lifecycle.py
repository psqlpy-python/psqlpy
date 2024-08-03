import asyncpg  # type: ignore[import-untyped]
import psqlpy
import psycopg_pool
from aiohttp import web

from psqlpy_stress.piccolo_conf import close_piccolo_pools, start_piccolo_pools
from psqlpy_stress.settings import DriversEnum, settings


async def startup(app: web.Application) -> None:
    await start_piccolo_pools()
    app[DriversEnum.PSQLPY] = psqlpy.ConnectionPool(
        dsn=settings.database_url,
        max_db_pool_size=settings.max_pool_size,
    )
    app[DriversEnum.ASYNCPG] = await asyncpg.create_pool(
        dsn=settings.database_url,
        min_size=1,
        max_size=settings.max_pool_size,
    )
    app[DriversEnum.PSYCOPG] = psycopg_pool.AsyncConnectionPool(
        conninfo=settings.database_url,
        max_size=settings.max_pool_size,
        min_size=1,
    )

    print("All pools started.")


async def shutdown(app: web.Application) -> None:
    await close_piccolo_pools()

    await app[DriversEnum.ASYNCPG].close()
    await app[DriversEnum.PSYCOPG].close()

    print("All pools closed.")
