---
title: Blacksheep
---

There is the default example for `Blacksheep` framework.

We strongly recommend to use the following example as a standard way to use `PSQLPy` with `Blacksheep` framework.

## Complete example

```python
# Start example
from __future__ import annotations

from typing import Any

import uvicorn
from blacksheep import Application, get
from psqlpy import ConnectionPool


app = Application()


@app.on_start
async def on_start(app: Application) -> None:
    """Create a database pool and saves it in the application state."""
    db_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/postgres",
        max_db_pool_size=10,
    )
    app.services.add_instance(db_pool)


@app.on_stop
async def on_stop(app: Application) -> None:
    """Close a database pool if it exists in app scope."""
    try:
        db_pool = app.services.resolve(ConnectionPool)
    except Exception:
        ...
    else:
        db_pool.close()


@get("/")
async def pg_pool_example(db_pool: ConnectionPool) -> list[dict[Any, Any]]:
    connection = await db_pool.connection()
    query_result = await connection.execute(
        "SELECT * FROM users",
    )
    return query_result.result()


if __name__ == "__main__":
    uvicorn.run(
        "start_example:app",
    )
```
