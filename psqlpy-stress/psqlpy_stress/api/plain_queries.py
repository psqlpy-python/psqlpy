import typing
import uuid

from aiohttp import web

from psqlpy_stress.settings import DriversEnum


if typing.TYPE_CHECKING:
    import asyncpg  # type: ignore[import-untyped]
    import psqlpy
    import psycopg_pool as psycopg_pool_package


async def psqlpy_simple_transaction_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    async with connection.transaction() as transaction:
        await transaction.execute("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def asyncpg_simple_transaction_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection, connection.transaction():
        await connection.fetch("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_simple_transaction_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection, connection.transaction():
        cursor = await connection.execute(
            "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


async def psqlpy_simple_connection_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.PSQLPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    await connection.execute(
        "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


async def asyncpg_simple_connection_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection:
        await connection.fetch("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_simple_connection_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        cursor = await connection.execute(
            "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


async def psqlpy_simple_pool_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.PSQLPool = request.app[DriversEnum.PSQLPY]
    await psqlpy_pool.execute(
        "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


async def asyncpg_simple_pool_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    await asyncpg_pool.fetch("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_simple_pool_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        cursor = await connection.execute(
            "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


# --------------------------------------------- Hard queries handlers starting here ---------------------------------------------


async def psqlpy_hard_transaction_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.PSQLPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    async with connection.transaction() as transaction:
        await transaction.execute(
            "SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10",
        )
    return web.Response(status=200, text="Ok")


async def asyncpg_hard_transaction_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection, connection.transaction():
        await connection.fetch("SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_hard_transaction_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection, connection.transaction():
        cursor = await connection.execute(
            "SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


async def psqlpy_hard_connection_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.PSQLPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    await connection.execute(
        "SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


async def asyncpg_hard_connection_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection:
        await connection.fetch("SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_hard_connection_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        cursor = await connection.execute(
            "SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


async def psqlpy_hard_pool_select(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.PSQLPool = request.app[DriversEnum.PSQLPY]
    await psqlpy_pool.execute(
        "SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


async def asyncpg_hard_pool_select(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    await asyncpg_pool.fetch("SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_hard_pool_select(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        cursor = await connection.execute(
            "SELECT * FROM big_table ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


# --------------------------------------------- Combined queries (select + insert) handlers starting here ---------------------------------------------


async def psqlpy_combined_transaction_query(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    async with connection.transaction() as transaction:
        await transaction.pipeline(
            [
                (
                    "INSERT INTO users (username) VALUES($1)",
                    [str(uuid.uuid4())],
                ),
                ("SELECT * FROM users ORDER BY RANDOM() LIMIT 10", []),
            ],
        )
    return web.Response(status=200, text="Ok")


async def asyncpg_combined_transaction_query(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection, connection.transaction():
        await connection.execute(
            "INSERT INTO users (username) VALUES($1)",
            str(uuid.uuid4()),
        )
        await connection.fetch("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_combined_transaction_query(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection, connection.transaction():
        await connection.execute(
            "INSERT INTO users (username) VALUES(%s)",
            [str(uuid.uuid4())],
        )
        cursor = await connection.execute(
            "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


async def psqlpy_combined_connection_query(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    connection = await psqlpy_pool.connection()
    await connection.execute(
        "INSERT INTO users (username) VALUES($1)",
        [str(uuid.uuid4())],
    )
    await connection.execute(
        "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        prepared=None,
        parameters=None,
    )
    return web.Response(status=200, text="Ok")


async def asyncpg_combined_connection_query(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    async with asyncpg_pool.acquire() as connection:
        await connection.execute(
            "INSERT INTO users (username) VALUES($1)",
            str(uuid.uuid4()),
        )
        await connection.fetch("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_combined_connection_query(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        await connection.execute(
            "INSERT INTO users (username) VALUES(%s)",
            [str(uuid.uuid4())],
        )
        cursor = await connection.execute(
            "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


async def psqlpy_combined_pool_query(request: web.Request) -> web.Response:
    psqlpy_pool: psqlpy.ConnectionPool = request.app[DriversEnum.PSQLPY]
    await psqlpy_pool.execute(
        "INSERT INTO users (username) VALUES($1)",
        parameters=[str(uuid.uuid4())],
    )
    await psqlpy_pool.execute("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def asyncpg_combined_pool_query(request: web.Request) -> web.Response:
    asyncpg_pool: asyncpg.Pool = request.app[DriversEnum.ASYNCPG]
    await asyncpg_pool.execute(
        "INSERT INTO users (username) VALUES($1)",
        str(uuid.uuid4()),
    )
    await asyncpg_pool.fetch("SELECT * FROM users ORDER BY RANDOM() LIMIT 10")
    return web.Response(status=200, text="Ok")


async def psycopg_combined_pool_query(request: web.Request) -> web.Response:
    psycopg_pool: psycopg_pool_package.AsyncConnectionPool = request.app[
        DriversEnum.PSYCOPG
    ]
    async with psycopg_pool.connection() as connection:
        await connection.execute(
            "INSERT INTO users (username) VALUES(%s)",
            [str(uuid.uuid4())],
        )
        cursor = await connection.execute(
            "SELECT * FROM users ORDER BY RANDOM() LIMIT 10",
        )
        await cursor.fetchall()
    return web.Response(status=200, text="Ok")


PLAIN_QUERY_ROUTES = [
    web.get(
        "/asyncpg-simple-transaction-select",
        asyncpg_simple_transaction_select,
    ),
    web.get(
        "/psqlpy-simple-transaction-select",
        psqlpy_simple_transaction_select,
    ),
    web.get(
        "/psycopg-simple-transaction-select",
        psycopg_simple_transaction_select,
    ),
    web.get(
        "/asyncpg-simple-connection-select",
        asyncpg_simple_connection_select,
    ),
    web.get(
        "/psqlpy-simple-connection-select",
        psqlpy_simple_connection_select,
    ),
    web.get(
        "/psycopg-simple-connection-select",
        psycopg_simple_connection_select,
    ),
    web.get(
        "/asyncpg-simple-pool-select",
        asyncpg_simple_pool_select,
    ),
    web.get(
        "/psqlpy-simple-pool-select",
        psqlpy_simple_pool_select,
    ),
    web.get(
        "/psycopg-simple-pool-select",
        psycopg_simple_pool_select,
    ),
    web.get(
        "/asyncpg-hard-transaction-select",
        asyncpg_hard_transaction_select,
    ),
    web.get(
        "/psqlpy-hard-transaction-select",
        psqlpy_hard_transaction_select,
    ),
    web.get(
        "/psycopg-hard-transaction-select",
        psycopg_hard_transaction_select,
    ),
    web.get(
        "/asyncpg-hard-connection-select",
        asyncpg_hard_connection_select,
    ),
    web.get(
        "/psqlpy-hard-connection-select",
        psqlpy_hard_connection_select,
    ),
    web.get(
        "/psycopg-hard-connection-select",
        psycopg_hard_connection_select,
    ),
    web.get(
        "/asyncpg-hard-pool-select",
        asyncpg_hard_pool_select,
    ),
    web.get(
        "/psqlpy-hard-pool-select",
        psqlpy_hard_pool_select,
    ),
    web.get(
        "/psycopg-hard-pool-select",
        psycopg_hard_pool_select,
    ),
    web.get(
        "/asyncpg-combined-transaction-query",
        asyncpg_combined_transaction_query,
    ),
    web.get(
        "/psqlpy-combined-transaction-query",
        psqlpy_combined_transaction_query,
    ),
    web.get(
        "/psycopg-combined-transaction-query",
        psycopg_combined_transaction_query,
    ),
    web.get(
        "/asyncpg-combined-connection-query",
        asyncpg_combined_connection_query,
    ),
    web.get(
        "/psqlpy-combined-connection-query",
        psqlpy_combined_connection_query,
    ),
    web.get(
        "/psycopg-combined-connection-query",
        psycopg_combined_connection_query,
    ),
    web.get(
        "/asyncpg-combined-pool-query",
        asyncpg_combined_pool_query,
    ),
    web.get(
        "/psqlpy-combined-pool-query",
        psqlpy_combined_pool_query,
    ),
    web.get(
        "/psycopg-combined-pool-query",
        psycopg_combined_pool_query,
    ),
]
