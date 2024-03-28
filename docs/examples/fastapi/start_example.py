# Start example
from contextlib import asynccontextmanager
from typing import AsyncGenerator, cast

import uvicorn
from fastapi import Depends, FastAPI, Request
from fastapi.responses import JSONResponse
from typing_extensions import Annotated

from psqlpy import Connection, PSQLPool


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Startup database connection pool and close it on shutdown."""
    db_pool = PSQLPool(
        dsn="postgres://postgres:postgres@localhost:5432/postgres",
        max_db_pool_size=2,
    )
    app.state.db_pool = db_pool
    yield
    await db_pool.close()


app = FastAPI(lifespan=lifespan)


async def db_connection(request: Request) -> Connection:
    """Retrieve new connection from connection pool and return it."""
    return await cast(PSQLPool, request.app.state.db_pool).connection()


@app.get("/")
async def pg_pool_example(
    db_connection: Annotated[Connection, Depends(db_connection)],
) -> JSONResponse:
    query_result = await db_connection.execute(
        "SELECT * FROM users",
    )
    return JSONResponse(content=query_result.result())


if __name__ == "__main__":
    uvicorn.run(
        "start_example:app",
    )
