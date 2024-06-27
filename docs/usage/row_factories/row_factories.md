---
title: Row Factories Usage
---

`row_factory` must be used when you want to process result from Database in a custom way and return something different from dictionary.

`row_factory` requires a function that accepts parameter `Dict[str, typing.Any]` and can return anything you want.

::: tip
`row_factory` can be a function or a class with `__call__` method which returns target converted instance.
:::

### Example:
We create custom class and function with this class as a parameter and return function which will be used in processing row from database.
```python
@dataclass
class ValidationTestModel:
    id: int
    name: str

def to_class(
    class_: Type[ValidationTestModel],
) -> Callable[[Dict[str, Any]], ValidationTestModel]:
    def to_class_inner(row: Dict[str, Any]) -> ValidationTestModel:
        return class_(**row)

    return to_class_inner

async def main() -> None:
    conn_result = await psql_pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    class_res = conn_result.row_factory(row_factory=to_class(ValidationTestModel))

    assert isinstance(class_res[0], ValidationTestModel)
```
