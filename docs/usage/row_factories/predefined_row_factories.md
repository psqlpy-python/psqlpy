---
title: Predefined row factories
---
We have predefined row_factories for fast usage.

### tuple_row
Instead of dict you can return tuple as a result.

```python
from psqlpy.row_factories import tuple_row

...


async def main() -> None:
    conn_result = await psql_pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    tuple_res = conn_result.row_factory(row_factory=tuple_row)

    assert isinstance(tuple_res[0], tuple)
```

### class_row
You can build class from database result.
```python
from dataclasses import dataclass

from psqlpy.row_factories import class_row

...


@dataclass
class ValidationTestModel:
    id: int
    name: str


async def main() -> None:
    conn_result = await psql_pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    tuple_res = conn_result.row_factory(row_factory=class_row(ValidationTestModel))

    assert isinstance(tuple_res[0], ValidationTestModel)
```