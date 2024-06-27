---
title: Results
---

`Results` are objects that driver returns to python with some data inside.

Currently there are two results:

- `QueryResult` - for result with multiple rows
- `SingleQueryResult` - for result with exactly one row

## QueryResult methods

### Result

#### Parameters
- `custom_decoders`: custom decoders for unsupported types. [Read more](./../../usage/types/advanced_type_usage.md)

Get the result as a list of dicts

```python
async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()
    query_result: QueryResult = await connection.execute(
        "SELECT username FROM users",
        [],
    )

    result: List[Dict[str, Any]] = query_result.result()
```

### As class

#### Parameters
- `as_class`: Custom class from Python.
- `custom_decoders`: custom decoders for unsupported types. [Read more](./../../usage/types/advanced_type_usage.md)

Get the result as a list of passed class instances.
Passed class can easily be either pydantic or msgspec model.

```python
class ExampleOfAsClass:
    def __init__(self, username: str) -> None:
        self.username = username


async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()
    query_result: QueryResult = await connection.execute(
        "SELECT username FROM users",
        [],
    )

    class_results: List[ExampleOfAsClass] = query_result.as_class(
        as_class=ExampleOfAsClass,
    )
```

### Row Factory

#### Parameters
- `row_factory`: custom callable object.
- `custom_decoders`: custom decoders for unsupported types. [Read more](./../../usage/types/advanced_type_usage.md)

[Read more](./../../usage/row_factories/overall_usage.md)

## SingleQueryResult methods

### Result

#### Parameters
- `custom_decoders`: custom decoders for unsupported types. [Read more](./../../usage/types/advanced_type_usage.md)

Get the result as a dict

```python
async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()
    query_result: SingleQueryResult = await db_pool.fetch_row(
        "SELECT username FROM users WHERE id = $1",
        [100],
    )

    result: Dict[str, Any] = query_result.result()
```

### As class

#### Parameters
- `as_class`: Custom class from Python.
- `custom_decoders`: custom decoders for unsupported types. [Read more](./../../usage/types/advanced_type_usage.md)

Get the result as a passed class instance.
Passed class can easily be either pydantic or msgspec model.

```python
class ExampleOfAsClass:
    def __init__(self, username: str) -> None:
        self.username = username


async def main() -> None:
    db_pool = ConnectionPool()
    connection = await db_pool.connection()
    query_result: SingleQueryResult = await connection.fetch_row(
        "SELECT username FROM users WHERE id = $1",
        [100],
    )
    class_results: ExampleOfAsClass = query_result.as_class(
        as_class=ExampleOfAsClass,
    )
```

### Row Factory

#### Parameters
- `row_factory`: custom callable object.
- `custom_decoders`: custom decoders for unsupported types. [Read more](./../../usage/types/advanced_type_usage.md)

[Read more](./../../usage/row_factories/overall_usage.md)
