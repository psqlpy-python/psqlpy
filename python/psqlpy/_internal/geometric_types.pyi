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
        - sequence of two sequences, each with two float numbers
        - sequence of four float

        ### Parameters:
        - `value`: any valid sequence with four float numbers.
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
        - sequence of sequences, each with two float numbers
        - sequence of float numbers which amount must be a multiple of two

        ### Parameters:
        - `value`: any valid structure with float numbers.
        """
