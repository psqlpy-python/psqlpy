---
title: Panther
---

There is the default example for `Panther` framework.

We strongly recommend to use the following example as a standard way to use `PSQLPy` with `Panther` framework.

## Complete example

```python
import uvicorn
from psqlpy import ConnectionPool

from panther import Panther
from panther.app import API
from panther.configs import config
from panther.events import Event
from panther.response import Response


@Event.startup
async def create_connection_pool():
    config.connection_pool = ConnectionPool(
        dsn="postgres://postgres:postgres@localhost:5432/postgres",
        max_db_pool_size=10,
    )


@Event.shutdown
async def close_connection_pool():
    config.connection_pool.close()


@API()
async def pg_pool_example():
    connection = await config.connection_pool.connection()
    query_result = await connection.execute(
        "SELECT * FROM users",
    )
    return Response(data=query_result.result())


app = Panther(__name__, configs=__name__, urls={'/': pg_pool_example})

if __name__ == "__main__":
    uvicorn.run(app)
```