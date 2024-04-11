import typing
import uuid

from aiohttp import web

from psqlpy_stress.influx_db_helpers import write_timings_to_influx
from psqlpy_stress.settings import DriversEnum


if typing.TYPE_CHECKING:
    import asyncpg  # type: ignore[import-untyped]
    import psqlpy
    import psycopg_pool as psycopg_pool_package


@write_timings_to_influx(DriversEnum.PSQLPY)
async def psqlpy_simple_transaction_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    async with connection.transaction() as transaction:
        await transaction.execute("SELECT * FROM users LIMIT 10")
    return web.Response(status=200, text=str(uuid.uuid4()))


@write_timings_to_influx(DriversEnum.ASYNCPG)
async def asyncpg_simple_transaction_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection, connection.transaction():
        await connection.fetch("SELECT * FROM users LIMIT 10")
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.PSYCOPG)
async def psycopg_simple_transaction_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection, connection.transaction():
        cursor = await connection.execute("SELECT * FROM users LIMIT 10")
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.PSQLPY)
async def psqlpy_simple_connection_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    await connection.execute(
        "SELECT * FROM users LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.ASYNCPG)
async def asyncpg_simple_connection_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection:
        await connection.fetch("SELECT * FROM users LIMIT 10")
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.PSYCOPG)
async def psycopg_simple_connection_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        cursor = await connection.execute("SELECT * FROM users LIMIT 10")
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.PSQLPY)
async def psqlpy_simple_pool_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    await psqlpy_pool.execute(
        "SELECT * FROM users LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.ASYNCPG)
async def asyncpg_simple_pool_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    await asyncpg_pool.fetch("SELECT * FROM users LIMIT 10")
    return web.Response(status=200, text="Ok")


@write_timings_to_influx(DriversEnum.PSYCOPG)
async def psycopg_simple_pool_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        cursor = await connection.execute("SELECT * FROM users LIMIT 10")
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")
