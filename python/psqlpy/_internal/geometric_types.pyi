from typing import Sequence, Union

from typing_extensions import Self

class PyPoint:
    """Represent point field in PostgreSQL and Point in Rust."""

    def __init__(
        self: Self,
        value: Sequence[float],
    ) -> None:
        """Create new instance of PyPoint.

        It accepts any sequence of two float numbers.

        ### Parameters:
        - `value`: sequence of two float numbers.
        """

class PyBox:
    """Represent box field in PostgreSQL and Rect in Rust."""

    def __init__(
        self: Self,
        value: Union[
            Sequence[Sequence[float]],
            Sequence[float],
        ],
    ) -> None:
        """Create new instance of PyBox.

        You need to pass any of this structures:
        - sequence of two sequences, each with pair of float numbers
        - sequence of two pairs of float

        ### Parameters:
        - `value`: any valid sequence with two pairs of float numbers.
        """

class PyPath:
    """Represent path field in PostgreSQL and LineString in Rust."""

    def __init__(
        self: Self,
        value: Union[
            Sequence[Sequence[float]],
            Sequence[float],
        ],
    ) -> None:
        """Create new instance of PyPath.

        You need to pass any of this structures:
        - sequence of sequences, each with pair of float numbers
        - sequence with pairs of float numbers

        ### Parameters:
        - `value`: any valid structure with float numbers.
        """

class PyLine:
    """Represent line field in PostgreSQL and Line in Rust."""

    def __init__(
        self: Self,
        value: Union[
            Sequence[Sequence[float]],
            Sequence[float],
        ],
    ) -> None:
        """Create new instance of PyLine.

        You need to pass any of this structures:
        - sequence of three float numbers
        - sequence of two sequences, each with pair of float numbers
        - sequence with two pairs of float numbers

        ### Parameters:
        - `value`: any valid structure with float numbers.
        """

class PyLineSegment:
    """Represent lseg field in PostgreSQL and Line in Rust."""

    def __init__(
        self: Self,
        value: Union[
            Sequence[Sequence[float]],
            Sequence[float],
        ],
    ) -> None:
        """Create new instance of PyLineSegment.

        You need to pass any of this structures:
        - sequence of two sequences, each with pair of float numbers
        - sequence with two pairs of float numbers

        ### Parameters:
        - `value`: any valid structure with float numbers.
        """

class PyPolygon:
    """Represent polygon field in PostgreSQL and Polygon in Rust."""

    def __init__(
        self: Self,
        value: Union[
            Sequence[Sequence[float]],
            Sequence[float],
        ],
    ) -> None:
        """Create new instance of PyPolygon.

        You need to pass any of this structures:
        - sequence of sequences, each with pair of float numbers
        - sequence with pairs of float numbers

        ### Parameters:
        - `value`: any valid structure with float numbers.
        """
