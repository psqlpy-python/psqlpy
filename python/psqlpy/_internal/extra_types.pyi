import typing
from datetime import date, datetime, time
from decimal import Decimal
from ipaddress import IPv4Address, IPv6Address
from uuid import UUID

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

class BoolArray:
    """Represent BOOLEAN ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                bool,
                typing.Sequence[bool],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of BoolArray.

        ### Parameters:
        - `inner`: inner value, sequence of UUID values.
        """

class UUIDArray:
    """Represent UUID ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                UUID,
                typing.Sequence[UUID],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of UuidArray.

        ### Parameters:
        - `inner`: inner value, sequence of UUID values.
        """

class VarCharArray:
    """Represent VarChar ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                str,
                typing.Sequence[str],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of VarCharArray.

        ### Parameters:
        - `inner`: inner value, sequence of str values.
        """

class TextArray:
    """Represent Text ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                str,
                typing.Sequence[str],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of TextArray.

        ### Parameters:
        - `inner`: inner value, sequence of str values.
        """

class Int16Array:
    """Represent INT2 ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                int,
                typing.Sequence[int],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of Int16Array.

        ### Parameters:
        - `inner`: inner value, sequence of int values.
        """

class Int32Array:
    """Represent INT4 ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                int,
                typing.Sequence[int],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of Int32Array.

        ### Parameters:
        - `inner`: inner value, sequence of int values.
        """

class Int64Array:
    """Represent INT8 ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                int,
                typing.Sequence[int],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of Int64Array.

        ### Parameters:
        - `inner`: inner value, sequence of int values.
        """

class Float32Array:
    """Represent FLOAT4 ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                float,
                typing.Sequence[float],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of Float32Array.

        ### Parameters:
        - `inner`: inner value, sequence of float values.
        """

class Float64Array:
    """Represent FLOAT8 ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                float,
                typing.Sequence[float],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of Float64Array.

        ### Parameters:
        - `inner`: inner value, sequence of float values.
        """

class MoneyArray:
    """Represent MONEY ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                int,
                typing.Sequence[int],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of MoneyArray.

        ### Parameters:
        - `inner`: inner value, sequence of int values.
        """

class IpAddressArray:
    """Represent INET ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                IPv4Address,
                IPv6Address,
                typing.Sequence[IPv4Address],
                typing.Sequence[IPv6Address],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of IpAddressArray.

        ### Parameters:
        - `inner`: inner value, sequence of IPv4Address/IPv6Address values.
        """

class JSONBArray:
    """Represent JSONB ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                typing.Dict[str, typing.Any],
                PyJSONB,
                typing.Sequence[typing.Dict[str, typing.Any]],
                typing.Sequence[PyJSONB],
                typing.Sequence[typing.Any],
            ]
        ],
    ) -> None:
        """Create new instance of JSONBArray.

        ### Parameters:
        - `inner`: inner value, sequence of values.
        """

class JSONArray:
    """Represent JSON ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                typing.Dict[str, typing.Any],
                PyJSON,
                typing.Sequence[typing.Dict[str, typing.Any]],
                typing.Sequence[PyJSON],
                typing.Sequence[typing.Any],
            ]
        ],
    ) -> None:
        """Create new instance of JSONArray.

        ### Parameters:
        - `inner`: inner value, sequence of values.
        """

class DateArray:
    """Represent DATE ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                date,
                typing.Sequence[date],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of DateArray.

        ### Parameters:
        - `inner`: inner value, sequence of date values.
        """

class TimeArray:
    """Represent TIME ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                time,
                typing.Sequence[time],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of DateArray.

        ### Parameters:
        - `inner`: inner value, sequence of time values.
        """

class DateTimeArray:
    """Represent TIMESTAMP ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                datetime,
                typing.Sequence[datetime],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of DateArray.

        ### Parameters:
        - `inner`: inner value, sequence of datetime values.
        """

class DateTimeTZArray:
    """Represent TIMESTAMPTZ ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                datetime,
                typing.Sequence[datetime],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of DateArray.

        ### Parameters:
        - `inner`: inner value, sequence of datetime values.
        """

class MacAddr6Array:
    """Represent MACADDR ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyMacAddr6,
                typing.Sequence[PyMacAddr6],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of MacAddr6Array.

        ### Parameters:
        - `inner`: inner value, sequence of PyMacAddr6 values.
        """

class MacAddr8Array:
    """Represent MACADDR8 ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyMacAddr8,
                typing.Sequence[PyMacAddr8],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of MacAddr8Array.

        ### Parameters:
        - `inner`: inner value, sequence of PyMacAddr8 values.
        """

class NumericArray:
    """Represent NUMERIC ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                Decimal,
                typing.Sequence[Decimal],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of NumericArray.

        ### Parameters:
        - `inner`: inner value, sequence of Decimal values.
        """

class PointArray:
    """Represent POINT ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyPoint,
                typing.Sequence[PyPoint],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of PointArray.

        ### Parameters:
        - `inner`: inner value, sequence of PyPoint values.
        """

class BoxArray:
    """Represent BOX ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyBox,
                typing.Sequence[PyBox],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of BoxArray.

        ### Parameters:
        - `inner`: inner value, sequence of PyBox values.
        """

class PathArray:
    """Represent PATH ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyPath,
                typing.Sequence[PyPath],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of PathArray.

        ### Parameters:
        - `inner`: inner value, sequence of PyPath values.
        """

class LineArray:
    """Represent LINE ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyLine,
                typing.Sequence[PyLine],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of LineArray.

        ### Parameters:
        - `inner`: inner value, sequence of PyLine values.
        """

class LsegArray:
    """Represent LSEG ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyLineSegment,
                typing.Sequence[PyLineSegment],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of LsegArray.

        ### Parameters:
        - `inner`: inner value, sequence of PyLineSegment values.
        """

class CircleArray:
    """Represent CIRCLE ARRAY in PostgreSQL."""

    def __init__(
        self: Self,
        inner: typing.Sequence[
            typing.Union[
                PyCircle,
                typing.Sequence[PyCircle],
                typing.Any,
            ],
        ],
    ) -> None:
        """Create new instance of CircleArray.

        ### Parameters:
        - `inner`: inner value, sequence of PyLineSegment values.
        """
