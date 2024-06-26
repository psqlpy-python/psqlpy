from typing import Any, Generic, Tuple, Type, TypeVar

from typing_extensions import Self

_CustomClass = TypeVar(
    "_CustomClass",
)

def tuple_row(row: dict[str, Any]) -> Tuple[Tuple[str, Any]]:
    """Convert dict row into tuple row.

    ### Parameters:
    - `row`: row in dictionary.

    ### Returns:
    row as a tuple of tuples.

    ### Example:
    ```
    dict_ = {
        "psqlpy": "is",
        "postgresql": "driver",
    }
    # This function will convert this dict into:
    (("psqlpy", "is"), ("postgresql": "driver"))
    ```
    """

class class_row(Generic[_CustomClass]):  # noqa: N801
    """Row converter to specified class.

    ### Example:
    ```python
    from psqlpy.row_factories import class_row


    class ValidationModel:
        name: str
        views_count: int


    async def main:
        res = await db_pool.execute(
            "SELECT * FROM users",
        )

        results: list[ValidationModel] = res.row_factory(
            class_row(ValidationModel),
        )
    ```
    """

    def __init__(self: Self, class_: Type[_CustomClass]) -> None:
        """Construct new `class_row`.

        ### Parameters:
        - `class_`: class to transform row into.
        """
    def __call__(self, row: dict[str, Any]) -> _CustomClass:
        """Convert row into specified class.

        ### Parameters:
        - `row`: row in dictionary.

        ### Returns:
        Constructed specified class.
        """
