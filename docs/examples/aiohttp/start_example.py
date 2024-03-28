# Start example
import asyncio
from typing import Any, cast

from aiohttp import web

from psqlpy import PSQLPool


async def start_db_pool(app: web.Application) -> None:
    """Initialize database connection pool."""
    db_pool = PSQLPool(
        dsn="postgres://postgres:postgres@localhost:5432/postgres",
        max_db_pool_size=2,
    )

    app["db_pool"] = db_pool


async def stop_db_pool(app: web.Application) -> None:
    """Close database connection pool."""
    db_pool = cast(PSQLPool, app.db_pool)
    await db_pool.close()


async def pg_pool_example(request: web.Request) -> Any:
    db_pool = cast(PSQLPool, request.app["db_pool"])
    connection = await db_pool.connection()
    await asyncio.sleep(10)
    query_result = await connection.execute(
        "SELECT * FROM users",
    )
    dict_result = query_result.result()
    return web.json_response(
        data=dict_result,
    )


application = web.Application()
application.on_startup.append(start_db_pool)
application.add_routes([web.get("/", pg_pool_example)])


if __name__ == "__main__":
    web.run_app(application)
