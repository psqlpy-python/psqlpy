---
title: FastAPI
---

There is the default example for `FastAPI` framework.

## Standard example.

This code is perfect for situations when your endpoints don't have complex logic
like sending messages over network with some queues (`RabbitMQ`, `NATS`, `Kafka` and etc)
or making long calculations, so a connection won't idle to much.
You need to take this restrictions into account if you don't have external database connection pool
like `PGBouncer`.

```python
# Start example
from contextlib import asynccontextmanager
from typing import Annotated, AsyncGenerator, cast
from fastapi import Depends, FastAPI, Request
from fastapi.responses import JSONResponse
from psqlpy import ConnectionPool, Connection
import uvicorn


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Startup database connection pool and close it on shutdown."""
    db_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/postgres",
        max_db_pool_size=10,
    )
    app.state.db_pool = db_pool
    yield
    await db_pool.close()


app = FastAPI(lifespan=lifespan)


async def db_connection(request: Request) -> Connection:
    """Retrieve new connection from connection pool and return it."""
    return await (cast(ConnectionPool, request.app.state.db_pool)).connection()


@app.get("/")
async def pg_pool_example(
    db_connection: Annotated[Connection, Depends(db_connection)],
):
    query_result = await db_connection.execute(
        "SELECT * FROM users",
    )
    return JSONResponse(content=query_result.result())


if __name__ == "__main__":
    uvicorn.run(
        "start_example:app",
    )
```

## Advanced example

If you don't have external connection pool like `PGBouncer` and your application have a lot of endpoints with a lot of complex logic,
so it's better not to take a connection from a pool at the start of an endpoint execution (don't use `Depends()` like in the previous example), because it will be blocked until the end of the endpoint logic.
The main idea is take a connection from a pool only for code parts in which it will be used immediately.

```python
# Start example
from contextlib import asynccontextmanager
from typing import Annotated, AsyncGenerator, cast
from fastapi import Depends, FastAPI, Request
from fastapi.responses import JSONResponse
from psqlpy import ConnectionPool, Connection
import uvicorn


db_pool = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=2,
)


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Startup database connection pool and close it on shutdown."""
    app.state.db_pool = db_pool
    yield
    db_pool.close()


app = FastAPI(lifespan=lifespan)


async def some_long_func() -> None:
    # Some very long execution.
    ...


@app.get("/")
async def pg_pool_example():
    await some_long_func()
    db_connection = await db_pool.connection()
    query_result = await db_connection.execute(
        "SELECT * FROM users",
    )
    return JSONResponse(content=query_result.result())


if __name__ == "__main__":
    uvicorn.run(
        "start_example:app",
    )
```
