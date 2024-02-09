from __future__ import annotations
from typing_extensions import Self


class SmallInt:
    """Represent SmallInt in PostgreSQL and `i16` in Rust."""
    
    def __init__(self: Self, inner_value: int) -> Self:
        """Create new instance of class.
        
        ### Parameters:
        - `inner_value`: int object.
        """


class Integer:
    """Represent Integer in PostgreSQL and `i32` in Rust."""
    
    def __init__(self: Self, inner_value: int) -> Self:
        """Create new instance of class.
        
        ### Parameters:
        - `inner_value`: int object.
        """


class BigInt:
    """Represent BigInt in PostgreSQL and `i64` in Rust."""
    
    def __init__(self: Self, inner_value: int) -> Self:
        """Create new instance of class.
        
        ### Parameters:
        - `inner_value`: int object.
        """
