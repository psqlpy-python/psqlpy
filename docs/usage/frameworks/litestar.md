---
title: Litestar
---

There is the default example for `Litestar` framework.

We strongly recommend to use the following example as a standard way to use `PSQLPy` with `Litestar` framework.

## Complete example

```python
# Start example
from __future__ import annotations

from typing import Any, cast

import uvicorn
from litestar import Litestar, Request, get
from psqlpy import ConnectionPool


def start_db_pool(app: Litestar) -> ConnectionPool:
    """Return the db pool.

    If it doesn't exist, creates it and saves it in on the application state object
    """
    if not getattr(app.state, "db_pool", None):
        app.state.db_pool = ConnectionPool(
            dsn="postgres://postgres:postgres@localhost:5432/postgres",
            max_db_pool_size=10,
        )

    return cast("ConnectionPool", app.state.db_pool)


async def stop_db_pool(app: Litestar) -> None:
    """Close database connection pool."""
    if getattr(app.state, "engine", None):
        db_pool = cast(ConnectionPool, app.state.db_pool)
        db_pool.close()


@get("/")
async def pg_pool_example(request: Request) -> list[dict[Any, Any]]:
    db_pool = cast(ConnectionPool, request.app.state.db_pool)
    connection = await db_pool.connection()
    query_result = await connection.execute(
        "SELECT * FROM users",
    )
    return query_result.result()


app = Litestar(
    [pg_pool_example],
    on_startup=[start_db_pool],
    on_shutdown=[stop_db_pool],
)


if __name__ == "__main__":
    uvicorn.run(
        "start_example:app",
    )

```
