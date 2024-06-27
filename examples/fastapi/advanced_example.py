# Start example
import asyncio
from contextlib import asynccontextmanager
from typing import AsyncGenerator

import uvicorn
from fastapi import FastAPI
from fastapi.responses import JSONResponse

from psqlpy import PSQLPool

db_pool = PSQLPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=2,
)


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Startup database connection pool and close it on shutdown."""
    app.state.db_pool = db_pool
    yield
    await db_pool.close()


app = FastAPI(lifespan=lifespan)


async def some_long_func() -> None:
    # Some very long execution.
    print("Executing...")
    await asyncio.sleep(10)
    print("Done.")


@app.get("/")
async def pg_pool_example() -> JSONResponse:
    await some_long_func()
    db_connection = await db_pool.connection()
    query_result = await db_connection.execute(
        "SELECT * FROM users",
    )
    return JSONResponse(content=query_result.result())


if __name__ == "__main__":
    uvicorn.run(
        "advanced_example:app",
        port=8001,
    )
