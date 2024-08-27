import typing

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
        value: typing.Union[
            dict[str, typing.Any],
            list[dict[str, typing.Any]],
            list[typing.Any],
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
        value: typing.Union[
            dict[str, typing.Any],
            list[dict[str, typing.Any]],
            list[typing.Any],
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

Coordinates: typing.TypeAlias = typing.Union[
    list[int | float],
    set[int | float],
    tuple[int | float, int | float],
]
PairsOfCoordinates: typing.TypeAlias = typing.Union[
    list[Coordinates | int | float],
    set[Coordinates | int | float],
    tuple[Coordinates | int | float, ...],
]

class PyPoint:
    """Represent point field in PostgreSQL and Point in Rust."""

    def __init__(self: Self, value: Coordinates) -> None:
        """Create new instance of PyPoint.

        It accepts any pair(List, Tuple or Set)
            of int/float numbers in every combination.

        ### Parameters:
        - `value`: pair of int/float numbers in every combination.
        """

class PyBox:
    """Represent box field in PostgreSQL and Rect in Rust."""

    def __init__(self: Self, value: PairsOfCoordinates) -> None:
        """Create new instance of PyBox.

        You need to pass any of this structures:
        - sequence(List, Tuple or Set) of two sequences(List, Tuple or Set),
            each with pair of int/float numbers in every combination
        - sequence(List, Tuple or Set) of two pairs of int/float in every combination

        ### Parameters:
        - `value`: any valid sequence(List, Tuple or Set) with two pairs
            of int/float numbers in every combination.
        """

class PyPath:
    """Represent path field in PostgreSQL and LineString in Rust."""

    def __init__(self: Self, value: PairsOfCoordinates) -> None:
        """Create new instance of PyPath.

        You need to pass any of this structures:
        - sequence(List, Tuple or Set) of sequences(List, Tuple or Set),
            each with pair of int/float numbers in every combination
        - sequence(List, Tuple or Set) with pairs
            of int/float numbers in every combination

        ### Parameters:
        - `value`: any valid structure with int/float numbers in every combination.
        """

class PyLine:
    """Represent line field in PostgreSQL and LineSegment in Rust."""

    def __init__(self: Self, value: PairsOfCoordinates) -> None:
        """Create new instance of PyLine.

        You need to pass any of this structures:
        - sequence of three int/float numbers(a, b, c)

        ### Parameters:
        - `value`: any valid structure with int/float numbers.
        """

class PyLineSegment:
    """Represent lseg field in PostgreSQL and LineSegment in Rust."""

    def __init__(self: Self, value: PairsOfCoordinates) -> None:
        """Create new instance of PyLineSegment.

        You need to pass any of this structures:
        - sequence(List, Tuple or Set) of two sequences(List, Tuple or Set),
            each with pair of int/float numbers in every combination
        - sequence(List, Tuple or Set) with two pairs
            of int/float numbers in every combination

        ### Parameters:
        - `value`: any valid structure with int/float numbers in every combination.
        """

class PyCircle:
    """Represent circle field in PostgreSQL and Circle in Rust."""

    def __init__(
        self: Self,
        value: typing.Union[
            list[int | float],
            set[int | float],
            tuple[int | float, int | float, int | float],
        ],
    ) -> None:
        """Create new instance of PyCircle.

        You need to pass any of this structures:
        - sequence of three int/float numbers(x, y, r)

        ### Parameters:
        - `value`: any valid structure with int/float numbers.
        """
