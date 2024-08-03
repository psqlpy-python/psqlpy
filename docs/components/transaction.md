---
title: Transaction
---

`Transaction` object represents `PostgreSQL` transaction.
There are two ways of how we can work with transactions on `PSQLPy` side.

### Transaction parameters

- `isolation_level`: level of isolation. Default how it is in PostgreSQL.
- `read_variant`: configure read variant of the transaction. Default how it is in PostgreSQL.
- `deferrable`: configure deferrable of the transaction. Default how it is in PostgreSQL.

### Control transaction fully on your own.

First of all, you can get transaction object only from connection object.

```python
from psqlpy import ConnectionPool


db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)


async def main() -> None:
    connection = await db_pool.connection()
    transaction = connection.transaction()
```

After this you need to start you transaction or in `PostgreSQL` terms you need to BEGIN it.

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()
```

So, after these manipulations you are ready to make you first query with the transaction.

```python
async def main() -> None:
    ...
    await transaction.execute(
        "INSERT INTO users (id, username) VALUES ($1, $2)",
        ["100", "Alex"],
    )
```

Good! We've inserted our first row, but if we won't commit the transaction all changes will discard.
::: warning
We need to commit changes.
:::

```python
async def main() -> None:
    ...
    await transaction.commit()
```

So, now everything is fine, changes are committed. But you can say that it's too complicated and you are right!
We have an alternative way to handle `begin()` and `commit()` automatically.

### Control transaction with async context manager.

There is the previous example but it is rewritten with use of async context manager.

```python
from psqlpy import ConnectionPool


db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)


async def main() -> None:
    await db_pool.startup()
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        # begin() calls automatically
        await transaction.execute(
            "INSERT INTO users (id, username) VALUES ($1, $2)",
            ["100", "Alex"],
        )
        # commit() calls automatically.
```

::: tip Cool tip
If a query raises an error in our async context manager, `ROLLBACK` is executed automatically.
:::
::: important
Transaction can be began only once, so if you have already called `begin()` manually then async context manager initialize will fail, you need to choose what to use.
:::

## Transaction methods

### Begin

You can start a transaction manually.

```python
async def main() -> None:
    ...
    await transaction.begin()
    ...
```

### Commit

You can commit a transaction manually.

```python
async def main() -> None:
    ...
    await transaction.commit()
    ...
```

### Execute

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of parameters for the statement string.
- `prepared`: Prepare statement before execution or not.

You can execute any query directly from `Transaction` object.
This method supports parameters, each parameter must be marked as `$<number>` (number starts with 1).

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        results: QueryResult = await transaction.execute(
            querystring="SELECT * FROM users WHERE id = $1 and username = $2",
            parameters=[100, "Alex"],
        )

    dict_results: list[dict[str, Any]] = results.result()
```

### Fetch

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of parameters for the statement string.
- `prepared`: Prepare statement before execution or not.

The same as the `execute` method, for some people this naming is preferable.

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        results: QueryResult = await transaction.fetch(
            querystring="SELECT * FROM users WHERE id = $1 and username = $2",
            parameters=[100, "Alex"],
        )

    dict_results: list[dict[str, Any]] = results.result()
```

### Execute Many

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of list of parameters for the statement string.
- `prepared`: Prepare statements before execution or not.

If you want to execute the same querystring, but with different parameters, `execute_many` is for you!

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        await transaction.execute_many(
            "INSERT INTO users (name, age) VALUES ($1, $2)",
            [["boba", 10], ["biba", 20]],
        )
```

### Fetch Row

#### Parameters

- `querystring`: Statement string.
- `parameters`: List of list of parameters for the statement string.
- `prepared`: Prepare statements before execution or not.

Sometimes you need to fetch only first row from the result.
::: warning
Querystring must return exactly one result or an exception will be raised.
:::

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        query_result: SingleQueryResult = await transaction.fetch_row(
            "SELECT username FROM users WHERE id = $1",
            [100],
        )
    dict_result: Dict[Any, Any] = query_result.result()
```

### Fetch Val

#### Parameters

- `querystring`: Statement string.
- `parameters`: List of list of parameters for the statement string.
- `prepared`: Prepare statements before execution or not.

If you need to retrieve some value not `QueryResult`.
::: warning
Querystring must return exactly one result or an exception will be raised.
:::

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        # this will be an int value
        query_result_value = await transaction.fetch_row(
            "SELECT COUNT(*) FROM users WHERE id > $1",
            [100],
        )
```

### Pipeline

#### Parameters

- `queries`: list of tuple. It must have structure like
- `prepared`: should the querystring/querystrings be prepared before the request. By default any querystrings will be prepared.

```python
queries = [
    ("SELECT * FROM users WHERE name = $1", ["some_name"]),
    ("SELECT 1", None),
]
```

- `prepared`: Prepare statements before execution or not.

Execute queries in pipeline.
Pipelining can improve performance in use cases in which multiple,
independent queries need to be executed.
In a traditional workflow,
each query is sent to the server after the previous query completes.
In contrast, pipelining allows the client to send all of the
queries to the server up front, minimizing time spent
by one side waiting for the other to finish sending data:

```
            Sequential                               Pipelined
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

[Read more!](https://docs.rs/tokio-postgres/latest/tokio_postgres/#pipelining)

Full example:

```python
import asyncio

from psqlpy import ConnectionPool, QueryResult


async def main() -> None:
    db_pool = ConnectionPool()
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction()

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

### Create Savepoint

#### Parameters:

- `savepoint_name`: name of the new savepoint.

Savepoint creation. [PostgreSQL docs](https://www.postgresql.org/docs/current/sql-savepoint.html)

```python
async def main() -> None:
    ...
    await transaction.create_savepoint("my_savepoint")
    await transaction.execute(...)
    await transaction.rollback_savepoint("my_savepoint")
```

### Rollback

Rollback the whole transaction. [PostgreSQL docs](https://www.postgresql.org/docs/current/sql-rollback.html)

```python
async def main() -> None:
    ...
    await transaction.execute(...)
    await transaction.rollback()
```

### Rollback Savepoint

#### Parameters:

- `savepoint_name`: name of the new savepoint.

Rollback to the specified savepoint. [PostgreSQL docs](https://www.postgresql.org/docs/current/sql-savepoint.html)

```python
async def main() -> None:
    ...
    transaction = connection.transaction()

    await transaction.create_savepoint("my_savepoint")
    await transaction.execute(...)
    await transaction.rollback_savepoint("my_savepoint")
```

### Release Savepoint

#### Parameters:

- `savepoint_name`: name of the new savepoint.

Release savepoint. [PostgreSQL docs](https://www.postgresql.org/docs/current/sql-savepoint.html)

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    transaction = connection.transaction()

    await transaction.create_savepoint("my_savepoint")
    await transaction.release_savepoint
```

### Cursor

#### Parameters

- `querystring`: Statement string.
- `parameters`: List of list of parameters for the statement string.
- `fetch_number`: rewrite default fetch_number. Default is 10.
- `scroll`: make cursor scrollable or not. Default is like in `PostgreSQL`.
- `prepared`: prepare querystring or not.

From `Transaction` you can create new `Cursor` object which represents cursor in the `PostgreSQL`. [PostgreSQL Docs](https://www.postgresql.org/docs/current/plpgsql-cursors.html)

```python
async def main() -> None:
    ...
    transaction = await connection.transaction()

    cursor = transaction.cursor(
        querystring="SELECT * FROM users WHERE username = $1",
        parameters=["Some_Username"],
        fetch_number=5,
    )
    await cursor.start()

    async for fetched_result in cursor:
        dict_result: List[Dict[Any, Any]] = fetched_result.result()
        ... # do something with the result.
```
