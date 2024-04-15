import asyncio

import uvloop
from aiohttp import web

from psqlpy_stress import api_transaction
from psqlpy_stress.lifecycle import shutdown, startup
from psqlpy_stress.settings import settings


asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())


app = web.Application()
app.on_startup.append(startup)
app.on_shutdown.append(shutdown)
app.add_routes(
    [
        web.get(
            "/asyncpg-simple-transaction-select",
            api_transaction.asyncpg_simple_transaction_select,
        ),
        web.get(
            "/psqlpy-simple-transaction-select",
            api_transaction.psqlpy_simple_transaction_select,
        ),
        web.get(
            "/psycopg-simple-transaction-select",
            api_transaction.psycopg_simple_transaction_select,
        ),
        web.get(
            "/asyncpg-simple-connection-select",
            api_transaction.asyncpg_simple_connection_select,
        ),
        web.get(
            "/psqlpy-simple-connection-select",
            api_transaction.psqlpy_simple_connection_select,
        ),
        web.get(
            "/psycopg-simple-connection-select",
            api_transaction.psycopg_simple_connection_select,
        ),
        web.get(
            "/asyncpg-simple-pool-select",
            api_transaction.asyncpg_simple_pool_select,
        ),
        web.get(
            "/psqlpy-simple-pool-select",
            api_transaction.psqlpy_simple_pool_select,
        ),
        web.get(
            "/psycopg-simple-pool-select",
            api_transaction.psycopg_simple_pool_select,
        ),
    ],
)


if __name__ == "__main__":
    web.run_app(app, port=settings.app_port, host="127.0.0.1")
