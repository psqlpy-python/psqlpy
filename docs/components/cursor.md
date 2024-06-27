---
title: Cursor
---

`Cursor` objects represents real `Cursor` in the `PostgreSQL`. [PostgreSQL docs](https://www.postgresql.org/docs/current/plpgsql-cursors.html)
It can be built only from `Transaction`.

## Cursor Parameters

- `querystring`: specify query for cursor.
- `parameters`: parameters for the querystring. Default `None`
- `fetch_number`: default fetch number. It is used in `fetch()` method and in async iterator. Default 10
- `scroll`: is cursor scrollable or not. Default as in `PostgreSQL`.

## Cursor as async iterator

The most common situation is using `Cursor` as async iterator.

```python
from psqlpy import ConnectionPool, QueryResult


async def main() -> None:
    db_pool = ConnectionPool()


    connection = await db_pool.connection()
    transaction = await connection.transaction()

    # Here we fetch 5 results in each iteration.
    async with cursor in transaction.cursor(
        querystring="SELECT * FROM users WHERE username = $1",
        parameters=["Some_Username"],
        fetch_number=5,
    ):
        async for fetched_result in cursor:
            dict_result: List[Dict[Any, Any]] = fetched_result.result()
            ... # do something with this result.
```

## Cursor methods

There are a lot of methods to work with cursor.

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

### Fetch

You can fetch next `N` records from the cursor.
It's possible to specify `N` fetch record with parameter `fetch_number`, otherwise will be used `fetch_number` from the `Cursor` initialization.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch(
        fetch_number=100,
    )
```

### Fetch Next

Just fetch next record from the `Cursor`.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_next()
```

### Fetch Prior

Just fetch previous record.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_prior()
```

### Fetch First

Just fetch the first record.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_first()
```

### Fetch Last

Just fetch the last record.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_last()
```

### Fetch Absolute

Just fetch absolute records.
It has `absolute_number` parameter, you must specify it.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_absolute(
        absolute_number=10,
    )
```

### Fetch Relative

Just fetch absolute records.
It has `relative_number` parameter, you must specify it.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_relative(
        relative_number=10,
    )
```

### Fetch Forward All

Fetch forward all records in the cursor.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_forward_all()
```

### Fetch Backward

Just backward records.
It has `backward_count` parameter, you must specify it.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_backward(
        backward_count=10,
    )
```

### Fetch Backward All

Fetch backward all records in the cursor.

```python
async def main() -> None:
    result: QueryResult = await cursor.fetch_backward_all()
```
