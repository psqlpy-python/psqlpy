[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/psqlpy?style=for-the-badge)](https://pypi.org/project/psqlpy/)
[![PyPI](https://img.shields.io/pypi/v/psqlpy?style=for-the-badge)](https://pypi.org/project/psqlpy/)
[![PyPI - Downloads](https://img.shields.io/pypi/dm/psqlpy?style=for-the-badge)](https://pypistats.org/packages/psqlpy)

# PSQLPy - Async PostgreSQL driver for Python written in Rust.

Driver for PostgreSQL written fully in Rust and exposed to Python.
The project is under active development and _**we cannot confirm that it's ready for production**_. Anyway, We will be grateful for the bugs found and open issues. Stay tuned.
_Normal documentation is in development._

## Installation

You can install package with `pip` or `poetry`.

poetry:

```bash
> poetry add psqlpy
```

pip:

```bash
> pip install psqlpy
```

Or you can build it by yourself. To do it, install stable rust and [maturin](https://github.com/PyO3/maturin).

```
> maturin develop --release
```

## Usage

Usage is as easy as possible.
Create new instance of PSQLPool, startup it and start querying.

```python
from typing import Any

from psqlpy import PSQLPool, QueryResult


db_pool = PSQLPool(
    username="postgres",
    password="pg_password",
    host="localhost",
    port=5432,
    db_name="postgres",
    max_db_pool_size=2,
)

async def main() -> None:
    await db_pool.startup()

    res: QueryResult = await db_pool.execute(
        "SELECT * FROM users",
    )

    print(res.result())
    # You don't need to close Database Pool by yourself,
    # rust does it instead.

```

Please take into account that each new execute gets new connection from connection pool.

### DSN support

You can separate specify `host`, `port`, `username`, etc or specify everything in one `DSN`.
**Please note that if you specify DSN any other argument doesn't take into account.**

```py
from typing import Any

from psqlpy import PSQLPool, QueryResult


db_pool = PSQLPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=2,
)

async def main() -> None:
    await db_pool.startup()

    res: QueryResult = await db_pool.execute(
        "SELECT * FROM users",
    )

    print(res.result())
    # You don't need to close Database Pool by yourself,
    # rust does it instead.
```

### Control connection recycling

There are 3 available options to control how a connection is recycled - `Fast`, `Verified` and `Clean`.
As connection can be closed in different situations on various sides you can select preferable behavior of how a connection is recycled.

- `Fast`: Only run `is_closed()` when recycling existing connections.
- `Verified`: Run `is_closed()` and execute a test query. This is slower, but guarantees that the database connection is ready to
  be used. Normally, `is_closed()` should be enough to filter
  out bad connections, but under some circumstances (i.e. hard-closed
  network connections) it's possible that `is_closed()`
  returns `false` while the connection is dead. You will receive an error
  on your first query then.
- `Clean`: Like [`Verified`] query method, but instead use the following sequence of statements which guarantees a pristine connection:
  ```sql
  CLOSE ALL;
  SET SESSION AUTHORIZATION DEFAULT;
  RESET ALL;
  UNLISTEN *;
  SELECT pg_advisory_unlock_all();
  DISCARD TEMP;
  DISCARD SEQUENCES;
  ```
  This is similar to calling `DISCARD ALL`. but doesn't call
  `DEALLOCATE ALL` and `DISCARD PLAN`, so that the statement cache is not
  rendered ineffective.

## Results from querying
You have some options to get results from the query.
`execute()` method, for example, returns `QueryResult` and this class can be converted into `list` of `dict`s - `list[dict[Any, Any]]` or into any Python class (`pydantic` model, as an example).

Let's see some code:
```python
from typing import Any

from pydantic import BaseModel
from psqlpy import PSQLPool, QueryResult


class ExampleModel(BaseModel):
    id: int
    username: str


db_pool = PSQLPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=2,
)

async def main() -> None:
    await db_pool.startup()

    res: QueryResult = await db_pool.execute(
        "SELECT * FROM users",
    )

    pydantic_res: list[ExampleModel] = res.as_class(
        as_class=ExampleModel,
    )
```

## Query parameters

You can pass parameters into queries.
Parameters can be passed in any `execute` method as the second parameter, it must be a list.
Any placeholder must be marked with `$< num>`.

```python
    res: QueryResult = await db_pool.execute(
        "SELECT * FROM users WHERE user_id = $1 AND first_name = $2",
        [100, "RustDriver"],
    )
```

## Connection

You can work with connection instead of DatabasePool.

```python
from typing import Any

from psqlpy import PSQLPool, QueryResult


db_pool = PSQLPool(
    username="postgres",
    password="pg_password",
    host="localhost",
    port=5432,
    db_name="postgres",
    max_db_pool_size=2,
)

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()

    res: QueryResult = await connection.execute(
        "SELECT * FROM users",
    )

    print(res.result())
    # You don't need to close connection by yourself,
    # rust does it instead.
```

## Transactions

Of course it's possible to use transactions with this driver.
It's as easy as possible and sometimes it copies common functionality from PsycoPG and AsyncPG.

### Transaction parameters

In process of transaction creation it is possible to specify some arguments to configure transaction.

- `isolation_level`: level of the isolation. By default - `None`.
- `read_variant`: read option. By default - `None`.
- `deferrable`: deferrable option. By default - `None`.

### You can use transactions as async context managers

By default async context manager only begins and commits transaction automatically.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel, QueryResult


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        res: QueryResult = await transaction.execute(
            "SELECT * FROM users",
        )

    print(res.result())
    # You don't need to close Database Pool by yourself,
    # rust does it instead.
```

### Or you can control transaction fully on your own.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    await transaction.execute(
        "INSERT INTO users VALUES ($1)",
        ["Some data"],
    )
    # You must commit the transaction by your own
    # or your changes will be vanished.
    await transaction.commit()
    # You don't need to close Database Pool by yourself,
    # rust does it instead.
```

### Transactions can be rolled back

You must understand that rollback can be executed only once per transaction.
After it's execution transaction state changes to `done`.
If you want to use `ROLLBACK TO SAVEPOINT`, see below.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    await transaction.execute(
        "INSERT INTO users VALUES ($1)",
        ["Some data"],
    )
    await transaction.rollback()
```

### Transaction execute many

You can execute one statement with multiple pararmeters.
The query will be executed with all parameters or will not be executed at all.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    await transaction.execute_many(
        "INSERT INTO users VALUES ($1)",
        [["first-data"], ["second-data"], ["third-data"]],
    )
    await transaction.commit()
```

### Transaction fetch row

You can fetch first row.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    first_row = await transaction.fetch_row(
        "SELECT * FROM users WHERE user_id = $1",
        ["user-id"],
    )
    first_row_result = first_row.result()  # This will be a dict.
```

### Transaction pipelining
When you have a lot of independent queries and want to execute them concurrently, you can use `pipeline`.
Pipelining can improve performance in use cases in which multiple,
independent queries need to be executed.
In a traditional workflow,
each query is sent to the server after the previous query completes.
In contrast, pipelining allows the client to send all of the queries to the server up front,
minimizing time spent by one side waiting for the other to finish sending data:
```
           Sequential                              Pipelined
| Client         | Server          |    | Client         | Server          |
|----------------|-----------------|    |----------------|-----------------|
| send query 1   |                 |    | send query 1   |                 |
|                | process query 1 |    | send query 2   | process query 1 |
| receive rows 1 |                 |    | send query 3   | process query 2 |
| send query 2   |                 |    | receive rows 1 | process query 3 |
|                | process query 2 |    | receive rows 2 |                 |
| receive rows 2 |                 |    | receive rows 3 |                 |
| send query 3   |                 |
|                | process query 3 |
| receive rows 3 |                 |
```
Read more: https://docs.rs/tokio-postgres/latest/tokio_postgres/#pipelining

Let's see some code:
```python
import asyncio

from psqlpy import PSQLPool, QueryResult


async def main() -> None:
    db_pool = PSQLPool()
    await db_pool.startup()

    transaction = await db_pool.transaction()

    results: list[QueryResult] = await transaction.pipeline(
        queries=[
            (
                "SELECT username FROM users WHERE id = $1",
                [100],
            ),
            (
                "SELECT some_data FROM profiles",
                None,
            ),
            (
                "INSERT INTO users (username, id) VALUES ($1, $2)",
                ["PSQLPy", 1],
            ),
        ]
    )
```

### Transaction ROLLBACK TO SAVEPOINT

You can rollback your transaction to the specified savepoint, but before it you must create it.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    # Create new savepoint
    await transaction.savepoint("test_savepoint")

    await transaction.execute(
        "INSERT INTO users VALUES ($1)",
        ["Some data"],
    )
    # Rollback to specified SAVEPOINT.
    await transaction.rollback_to("test_savepoint")

    await transaction.commit()
```

### Transaction RELEASE SAVEPOINT

It's possible to release savepoint

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    # Create new savepoint
    await transaction.savepoint("test_savepoint")
    # Release savepoint
    await transaction.release_savepoint("test_savepoint")

    await transaction.commit()
```

## Cursors

Library supports PostgreSQL cursors.

Cursors can be created only in transaction. In addition, cursor supports async iteration.

### Cursor parameters

In process of cursor creation you can specify some configuration parameters.

- `querystring`: query for the cursor. Required.
- `parameters`: parameters for the query. Not Required.
- `fetch_number`: number of records per fetch if cursor is used as an async iterator. If you are using `.fetch()` method you can pass different fetch number. Not required. Default - 10.
- `scroll`: set `SCROLL` if True or `NO SCROLL` if False. Not required. By default - `None`.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel, QueryResult


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    # Create new savepoint
    cursor = await transaction.cursor(
        querystring="SELECT * FROM users WHERE username = $1",
        parameters=["SomeUserName"],
        fetch_number=100,
    )

    # You can manually fetch results from cursor
    results: QueryResult = await cursor.fetch(fetch_number=8)

    # Or you can use it as an async iterator.
    async for fetched_result in cursor:
        print(fetched_result.result())

    # If you want to close cursor, please do it manually.
    await cursor.close()

    await transaction.commit()
```

### Cursor operations

Available cursor operations:

- FETCH count - `cursor.fetch(fetch_number=)`
- FETCH NEXT - `cursor.fetch_next()`
- FETCH PRIOR - `cursor.fetch_prior()`
- FETCH FIRST - `cursor.fetch_first()`
- FETCH LAST - `cursor.fetch_last()`
- FETCH ABSOLUTE - `cursor.fetch_absolute(absolute_number=)`
- FETCH RELATIVE - `cursor.fetch_relative(relative_number=)`
- FETCH FORWARD ALL - `cursor.fetch_forward_all()`
- FETCH BACKWARD backward_count - `cursor.fetch_backward(backward_count=)`
- FETCH BACKWARD ALL - `cursor.fetch_backward_all()`

## Extra Types

Sometimes it's impossible to identify which type user tries to pass as a argument. But Rust is a strongly typed programming language so we have to help.

| Extra Type in Python | Type in PostgreSQL | Type in Rust |
| -------------------- | ------------------ | ------------ |
| SmallInt             | SmallInt           | i16          |
| Integer              | Integer            | i32          |
| BigInt               | BigInt             | i64          |
| PyUUID               | UUID               | Uuid         |
| PyJSON               | JSON, JSONB        | Value        |

```python
from typing import Any

import uuid

from psqlpy import PSQLPool

from psqlpy.extra_types import (
    SmallInt,
    Integer,
    BigInt,
    PyUUID,
    PyJSON,
)


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    res: list[dict[str, Any]] = await db_pool.execute(
        "INSERT INTO users VALUES ($1, $2, $3, $4, $5)",
        [
            SmallInt(100),
            Integer(10000),
            BigInt(9999999),
            PyUUID(uuid.uuid4().hex),
            PyJSON(
                [
                    {"we": "have"},
                    {"list": "of"},
                    {"dicts": True},
                ],
            )
        ]
    )

    print(res)
    # You don't need to close Database Pool by yourself,
    # rust does it instead.

```

## Benchmarks

We have made some benchmark to compare `PSQLPy`, `AsyncPG`, `Psycopg3`.
Main idea is do not compare clear drivers because there are a few situations in which you need to use only driver without any other dependencies.

**So infrastructure consists of:**

1. AioHTTP
2. PostgreSQL driver (`PSQLPy`, `AsyncPG`, `Psycopg3`)
3. PostgreSQL v15. Server is located in other part of the world, because we want to simulate network problems.
4. Grafana (dashboards)
5. InfluxDB
6. JMeter (for load testing)

The results are very promising! `PSQLPy` is faster than `AsyncPG` at best by 2 times, at worst by 45%. `PsycoPG` is 3.5 times slower than `PSQLPy` in the worst case, 60% in the best case.
