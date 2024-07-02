import asyncio

import uvloop
from aiohttp import web

from psqlpy_stress.api.piccolo import PICCOLO_QUERY_ROUTES
from psqlpy_stress.api.plain_queries import PLAIN_QUERY_ROUTES
from psqlpy_stress.lifecycle import shutdown, startup
from psqlpy_stress.settings import settings


asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())


app = web.Application()
app.on_startup.append(startup)
app.on_shutdown.append(shutdown)
app.add_routes(PLAIN_QUERY_ROUTES)
app.add_routes(PICCOLO_QUERY_ROUTES)


if __name__ == "__main__":
    web.run_app(app, port=settings.app_port, host="127.0.0.1")
