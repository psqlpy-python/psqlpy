from typing import Any, Union

from typing_extensions import Self

class SmallInt:
    """Represent SmallInt in PostgreSQL and `i16` in Rust."""

    def __init__(self: Self, inner_value: int) -> None:
        """Create new instance of class.

        ### Parameters:
        - `inner_value`: int object.
        """

class Integer:
    """Represent Integer in PostgreSQL and `i32` in Rust."""

    def __init__(self: Self, inner_value: int) -> None:
        """Create new instance of class.

        ### Parameters:
        - `inner_value`: int object.
        """

class BigInt:
    """Represent BigInt in PostgreSQL and `i64` in Rust."""

    def __init__(self: Self, inner_value: int) -> None:
        """Create new instance of class.

        ### Parameters:
        - `inner_value`: int object.
        """

class Money:
    """Represent `MONEY` in PostgreSQL and `i64` in Rust."""

    def __init__(self: Self, inner_value: int) -> None:
        """Create new instance of class.

        ### Parameters:
        - `inner_value`: int object.
        """

class Float32:
    """Represents `FLOAT4` in `PostgreSQL` and `f32` in Rust."""

    def __init__(self: Self, inner_value: float) -> None:
        """Create new instance of a class.

        ### Parameters:
        - `inner_value`: float object.
        """

class Float64:
    """Represents `FLOAT8` in `PostgreSQL` and `f64` in Rust."""

    def __init__(self: Self, inner_value: float) -> None:
        """Create new instance of a class.

        ### Parameters:
        - `inner_value`: float object.
        """

class PyVarChar:
    """Represent VarChar in PostgreSQL and String in Rust."""

    def __init__(self: Self, inner_value: str) -> None:
        """Create new instance of class.

        You need to pass uuid as a str.

        ### Parameters:
        - `inner_value`: str object.
        """

class PyText:
    """Represent TEXT in PostgreSQL and String ins Rust."""

    def __init__(self: Self, inner_value: str) -> None:
        """Create new instance of class.

        You need to pass uuid as a str.

        ### Parameters:
        - `inner_value`: str object.
        """

class PyJSONB:
    """Represent JSONB field in PostgreSQL and Value in Rust."""

    def __init__(
        self: Self,
        value: Union[
            dict[str, Any],
            list[dict[str, Any]],
        ],
    ) -> None:
        """Create new instance of PyJSON.B.

        It accepts structure that can be used in JSON/JSONB fields.

        ### Parameters:
        - `value`: value for the JSONB field.
        """

class PyJSON:
    """Represent JSON field in PostgreSQL and Value in Rust."""

    def __init__(
        self: Self,
        value: Union[
            dict[str, Any],
            list[dict[str, Any]],
        ],
    ) -> None:
        """Create new instance of PyJSON.

        It accepts structure that can be used in JSON/JSONB fields.

        ### Parameters:
        - `value`: value for the JSONB field.
        """

class PyMacAddr6:
    """Represents MACADDR in PostgreSQL."""

    def __init__(self, value: str) -> None:
        """Construct new MacAddr.

        ### Parameters:
        - `value`: value for MACADDR field.
        """

class PyMacAddr8:
    """Represents MACADDR8 in PostgreSQL."""

    def __init__(self, value: str) -> None:
        """Construct new MacAddr8.

        ### Parameters:
        - `value`: value for MACADDR8 field.
        """

class PyCustomType:
    def __init__(self, value: bytes) -> None: ...
