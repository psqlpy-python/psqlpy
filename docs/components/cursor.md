---
title: Cursor
---

`Cursor` objects represents server-side `Cursor` in the `PostgreSQL`. [PostgreSQL docs](https://www.postgresql.org/docs/current/plpgsql-cursors.html).
::: important
Cursor always lives inside a transaction. If you don't begin transaction explicitly, it will be opened anyway.
:::

## Cursor Parameters

- `querystring`: specify query for cursor.
- `parameters`: parameters for the querystring. Default `None`
- `fetch_number`: default fetch number. It is used in `fetch()` method and in async iterator.

## Usage

Cursor can be used in different ways.

::: tabs
@tab Pre-Initialization
```python
from psqlpy import ConnectionPool, QueryResult

async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()

    cursor = connection.cursor(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
        fetch_number=10,
    )
    await cursor.start()
```

@tab Post-Initialization
```python
from psqlpy import ConnectionPool, QueryResult

async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()

    cursor = connection.cursor()
    await cursor.execute(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )
    result: QueryResult = await cursor.fetchone()
```

@tab Async Context Manager
```python
from psqlpy import ConnectionPool, QueryResult

async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()

    async with connection.cursor(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
        array_size=10,
    ) as cursor:
        result: QueryResult = await cursor.fetchone()
```

@tab Async Iterator
```python
from psqlpy import ConnectionPool, QueryResult

async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()

    cursor = connection.cursor(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
        fetch_number=10,
    )
    await cursor.start()

    async for result in cursor:
        print(result)
```
:::

## Cursor attributes
- `array_size`: get and set attribute. Used in async iterator and `fetch_many` method.

## Cursor methods
### Start
Declare (create) cursor.

```python
async def main() -> None:
    await cursor.start()
```

### Close

Close the cursor

```python
async def main() -> None:
    await cursor.close()
```

### Execute

Initialize cursor and make it ready for fetching.

::: important
If you initialized cursor with `start` method or via context manager, you don't have to use this method.
:::

#### Parameters:
- `querystring`: specify query for cursor.
- `parameters`: parameters for the querystring. Default `None`

```python
async def main() -> None:
    await cursor.execute(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )
    result: QueryResult = await cursor.fetchone()
```

### Fetchone

Fetch one result from the cursor.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetchone()
```

### Fetchmany

Fetch N results from the cursor. Default is `array_size`.

#### Parameters:
- `size`: number of records to fetch.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetchmany(size=10)
```

### Fetchall

Fetch all results from the cursor.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetchall()
```
