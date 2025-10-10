---
title: Connection
---

`Connection` object represents single connection to the `PostgreSQL`. You must work with database within it.
`Connection` get be made with `ConnectionPool().connection()` method.

## Usage
::: tabs

@tab default
```python
from psqlpy import ConnectionPool

db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)

async def main() -> None:
    connection = await db_pool.connection()
```

@tab single connection
```python
from psqlpy import connect

async def main() -> None:
    db_connection: Final = await connect(
        dsn="postgres://postgres:postgres@localhost:5432/postgres",
    )
    await db_connection.execute(...)
```

@tab async context manager
```python
from psqlpy import ConnectionPool

db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)

async def main() -> None:
    async with db_pool.acquire() as connection:
        # connection is valid here
        ...
    # connection is invalid here
```
:::

## Connection methods

### Execute

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of parameters for the statement string.
- `prepared`: Prepare statement before execution or not.

You can execute any query directly from `Connection` object.
This method supports parameters, each parameter must be marked as `$<number>` in querystring (number starts with 1).

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    results: QueryResult = await connection.execute(
        "SELECT * FROM users WHERE id = $1 and username = $2",
        [100, "Alex"],
    )

    dict_results: list[dict[str, Any]] = results.result()
```

### Execute Batch

#### Parameters:

- `querystring`: querystrings separated by semicolons.

Executes a sequence of SQL statements using the simple query protocol.

Statements should be separated by semicolons.
If an error occurs, execution of the sequence will stop at that point.
This is intended for use when, for example,
initializing a database schema.

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    await connection.execute_batch(
        "CREATE TABLE psqlpy (name VARCHAR); CREATE TABLE psqlpy2 (name VARCHAR);",
    )
```

### Execute Many

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of list of parameters for the statement string.
- `prepared`: Prepare statement before execution or not.

This method supports parameters, each parameter must be marked as `$<number>` in querystring (number starts with 1).
Atomicity is provided, so you don't need to worry about unsuccessful result, because there is a transaction used internally.
This method returns nothing.

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    await connection.execute_many(
        "INSERT INTO users (name, age) VALUES ($1, $2)",
        [["boba", 10], ["boba", 20]],
    )
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
    results: QueryResult = await connection.fetch(
        "SELECT * FROM users WHERE id = $1 and username = $2",
        [100, "Alex"],
    )

    dict_results: list[dict[str, Any]] = results.result()
```

### Fetch Row

#### Parameters:

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
    # this will be an int value
    query_result_value = await connection.fetch_val(
        "SELECT COUNT(*) FROM users WHERE id > $1",
        [100],
    )
```

### Transaction

#### Parameters:

- `isolation_level`: level of isolation. Default how it is in PostgreSQL.
- `read_variant`: configure read variant of the transaction. Default how it is in PostgreSQL.
- `deferrable`: configure deferrable of the transaction. Default how it is in PostgreSQL.

```python
from psqlpy import IsolationLevel, ReadVariant

async def main() -> None:
    ...
    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
        read_variant=ReadVariant.ReadWrite,
        deferrable=True,
    )
```

### Cursor
Create new server-side cursor

#### Parameters
- `querystring`: querystring for cursor.
- `parameters`: parameters for querystring.
- `fetch_number`: default value for fetch number, can be changed.

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    cursor = connection.cursor(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
        fetch_number=5,
    )
```

### Prepare
Prepare statement and return new instance.

#### Parameters:
- `querystring`: querystring for statement.
- `parameters`: parameters for querystring.

```python
from psqlpy import IsolationLevel, ReadVariant

async def main() -> None:
    ...
    connection = await db_pool.connection()
    prepared_stmt = await connection.prepare(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )
```

### Close
Returns connection to the pool.
It's crucial to commit all transactions and close all cursor which are made from the connection.
Otherwise, this method won't do anything useful.

::: tip
There is no need in this method if you use async context manager.
:::

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    connection.close()
```
