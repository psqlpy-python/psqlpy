---
title: Advanced Type Usage
---
Due to an unavailability to support all possible types in PostgreSQL, we have a way to encode Python types into PostgreSQL ones and decode wise versa.

This section has `Advanced` in the name because you'll need to work with raw bytes which can be difficult for some developers.

## Pass unsupported type into PostgreSQL
If you are using some type that we don't support and want to insert it into PostgreSQL from PSQLPy, you must use `PyCustomType` class.

Let's assume we have table `for_test` in the database and `PSQLPy` doesn't support (only for demonstration) `VARCHAR` type:
|  database type | database column name |
| :---: | :---: |
| VARCHAR | nickname |
```python
from typing import Final

from psqlpy import ConnectionPool
from psqlpy.extra_types import PyCustomType


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    await db_pool.execute(
        "INSERT INTO for_test (nickname) VALUES ($1)",
        [PyCustomType(b"SomeDataInBytes")],
    )
    db_pool.close()
```

Here we pass `PyCustomType` into the parameters. It accepts only bytes.

::: important
You must make bytes passed into `PyCustomType` readable for `PostgreSQL`.
If bytes will be wrong, you will get an exception.
:::

## Decode unsupported type from PostgreSQL
When you retrieve some data from the `PostgreSQL` there are can be data types that we don't support yet.
To deal with this situation, you can use `custom_decoders` parameter in `result()` and `as_class()` methods.

Let's assume we have table `for_test` in the database and `PSQLPy` doesn't support (only for demonstration) `VARCHAR` type:
|  database type | database column name |
| :---: | :---: |
| VARCHAR | nickname |
```python
from typing import Final, Any

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import PyCustomType


def nickname_decoder(bytes_from_psql: bytes | None) -> str:
    return bytes_from_psql.decode() if bytes_from_psql else None


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    result: QueryResult = await db_pool.execute(
        "SELECT * FROM for_test",
        [PyCustomType(b"SomeDataInBytes")],
    )

    parsed_result: list[dict[str, Any]] = result.result(
        custom_decoders={
            "nickname": nickname_decoder,
        },
    )
    db_pool.close()
```

::: important
Rules about `custom_decoders` parameter:
- The key of the dict must be the name of the field on which you want to apply the decode function.
- If you use aliases for the result field name, you must specify the alias.
- The key of the dict must be in **lowercase**.
:::
