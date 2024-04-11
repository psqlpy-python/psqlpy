import asyncpg  # type: ignore[import-untyped]
import psqlpy
import psycopg_pool
from aiohttp import web
from influxdb_client.client.influxdb_client_async import InfluxDBClientAsync

from psqlpy_stress.settings import DriversEnum, settings


async def startup(app: web.Application) -> None:
    app[settings.influx_db_client_app_key] = InfluxDBClientAsync(
        url=settings.influx_db_address,
        token=settings.influx_db_token,
        org=settings.influx_db_organization,
    )

    app[DriversEnum.PSQLPY] = psqlpy.ConnectionPool(
        dsn=settings.database_url,
        max_db_pool_size=settings.max_pool_size,
    )
    app[DriversEnum.ASYNCPG] = await asyncpg.create_pool(
        dsn=settings.database_url,
        min_size=4,
        max_size=settings.max_pool_size,
    )
    app[DriversEnum.PSYCOPG] = psycopg_pool.AsyncConnectionPool(
        conninfo=settings.database_url,
        max_size=settings.max_pool_size,
    )

    print("All pools created")


async def shutdown(app: web.Application) -> None:
    await app[DriversEnum.PSQLPY].close()
    await app[DriversEnum.ASYNCPG].close()
    await app[DriversEnum.PSYCOPG].close()
    await app[settings.influx_db_client_app_key].close()

    print("All pools stopped")
