---
title: Robyn
---

There is the default example for `Robyn` framework.

We strongly recommend to use the following example as a standard way to use `PSQLPy` with `Robyn` framework.

## Complete example

```python
# Start example
from __future__ import annotations

import asyncio
from typing import Any

from psqlpy import ConnectionPool
from robyn import Request, Robyn

db_pool = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=10,
)

app = Robyn(__file__)


@app.get("/")
async def pg_pool_example(request: Request) -> list[dict[Any, Any]]:
    connection = await db_pool.connection()
    query_result = await connection.execute(
        "SELECT * FROM users",
    )
    return query_result.result()


async def main() -> None:
    try:
        app.start(host="127.0.0.1", port=8000)
    finally:
        db_pool.close()


if __name__ == "__main__":
    asyncio.run(main())
```
