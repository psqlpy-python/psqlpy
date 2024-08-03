import uuid

from aiohttp import web
from piccolo.engine import engine_finder
from piccolo.query import OrderByRaw

from psqlpy_stress.models.piccolo import SomeBigTable, User
from psqlpy_stress.piccolo_conf import (
    ASYNCPG_PICCOLO_ENGINE,
    PSQLPY_PICCOLO_ENGINE,
)


async def psqlpy_simple_transaction_select_piccolo(
    _request: web.Request,
) -> web.Response:
    async with PSQLPY_PICCOLO_ENGINE.transaction():
        User._meta.db = PSQLPY_PICCOLO_ENGINE
        await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def asyncpg_simple_transaction_select_piccolo(
    _request: web.Request,
) -> web.Response:
    async with ASYNCPG_PICCOLO_ENGINE.transaction():
        User._meta.db = ASYNCPG_PICCOLO_ENGINE
        await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def psqlpy_simple_connection_select_piccolo(_request: web.Request) -> web.Response:
    User._meta.db = PSQLPY_PICCOLO_ENGINE
    await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def asyncpg_simple_connection_select_piccolo(
    _request: web.Request,
) -> web.Response:
    User._meta.db = ASYNCPG_PICCOLO_ENGINE
    await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


# --------------------------------------------- Hard queries handlers starting here ---------------------------------------------


async def psqlpy_hard_transaction_select_piccolo(
    _request: web.Request,
) -> web.Response:
    async with PSQLPY_PICCOLO_ENGINE.transaction():
        SomeBigTable._meta.db = PSQLPY_PICCOLO_ENGINE
        await SomeBigTable.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def asyncpg_hard_transaction_select_piccolo(
    _request: web.Request,
) -> web.Response:
    engine_finder().set_engine(ASYNCPG_PICCOLO_ENGINE)
    async with ASYNCPG_PICCOLO_ENGINE.transaction():
        SomeBigTable._meta.db = ASYNCPG_PICCOLO_ENGINE
        await SomeBigTable.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def psqlpy_hard_connection_select_piccolo(_request: web.Request) -> web.Response:
    SomeBigTable._meta.db = PSQLPY_PICCOLO_ENGINE
    await SomeBigTable.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def asyncpg_hard_connection_select_piccolo(
    _request: web.Request,
) -> web.Response:
    SomeBigTable._meta.db = ASYNCPG_PICCOLO_ENGINE
    await SomeBigTable.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


# --------------------------------------------- Combined queries (select + insert) handlers starting here ---------------------------------------------


async def psqlpy_combined_transaction_query_piccolo(
    _request: web.Request,
) -> web.Response:
    async with PSQLPY_PICCOLO_ENGINE.transaction():
        User._meta.db = PSQLPY_PICCOLO_ENGINE
        await User.insert(User(username=str(uuid.uuid4())))
        await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def asyncpg_combined_transaction_query_piccolo(
    _request: web.Request,
) -> web.Response:
    engine_finder().set_engine(ASYNCPG_PICCOLO_ENGINE)
    async with ASYNCPG_PICCOLO_ENGINE.transaction():
        User._meta.db = ASYNCPG_PICCOLO_ENGINE
        await User.insert(User(username=str(uuid.uuid4())))
        await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def psqlpy_combined_connection_query_piccolo(
    _request: web.Request,
) -> web.Response:
    User._meta.db = PSQLPY_PICCOLO_ENGINE
    await User.insert(User(username=str(uuid.uuid4())))
    await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


async def asyncpg_combined_connection_query_piccolo(
    _request: web.Request,
) -> web.Response:
    User._meta.db = ASYNCPG_PICCOLO_ENGINE
    await User.insert(User(username=str(uuid.uuid4())))
    await User.select().order_by(OrderByRaw("random()")).limit(10)
    return web.Response(status=200, text="Ok")


PICCOLO_QUERY_ROUTES = [
    web.get(
        "/asyncpg-simple-transaction-select-piccolo",
        asyncpg_simple_transaction_select_piccolo,
    ),
    web.get(
        "/psqlpy-simple-transaction-select-piccolo",
        psqlpy_simple_transaction_select_piccolo,
    ),
    web.get(
        "/asyncpg-simple-connection-select-piccolo",
        asyncpg_simple_connection_select_piccolo,
    ),
    web.get(
        "/psqlpy-simple-connection-select-piccolo",
        psqlpy_simple_connection_select_piccolo,
    ),
    web.get(
        "/asyncpg-hard-transaction-select-piccolo",
        asyncpg_hard_transaction_select_piccolo,
    ),
    web.get(
        "/psqlpy-hard-transaction-select-piccolo",
        psqlpy_hard_transaction_select_piccolo,
    ),
    web.get(
        "/asyncpg-hard-connection-select-piccolo",
        asyncpg_hard_connection_select_piccolo,
    ),
    web.get(
        "/psqlpy-hard-connection-select-piccolo",
        psqlpy_hard_connection_select_piccolo,
    ),
    web.get(
        "/asyncpg-combined-transaction-query-piccolo",
        asyncpg_combined_transaction_query_piccolo,
    ),
    web.get(
        "/psqlpy-combined-transaction-query-piccolo",
        psqlpy_combined_transaction_query_piccolo,
    ),
    web.get(
        "/asyncpg-combined-connection-query-piccolo",
        asyncpg_combined_connection_query_piccolo,
    ),
    web.get(
        "/psqlpy-combined-connection-query-piccolo",
        psqlpy_combined_connection_query_piccolo,
    ),
]
