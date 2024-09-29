---
title: Frequently asked questions
---

Here you can find most common questions and problems.

### LIMIT of OFFSET isn't working
The main problem is PostgreSQL expects `LIMIT` and `OFFSET` to be BIGINT type but when you pass python `int` into `parameters` it converts to `INTEGER`.

#### Problem and Solution:
```python
from psqlpy import ConnectionPool
from psqlpy.extra_types import BigInt

# --- Incorrect ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="SELECT * FROM users LIMIT $1 OFFSET $2",
        parameters=[10, 100],
    )


# --- Correct ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="SELECT * FROM users LIMIT $1 OFFSET $2",
        parameters=[BigInt(10), BigInt(100)],
    )
```

### WHERE IN clause isn't working
Instead of using `WHERE <field> IN ()` clause you must use `WHERE <field> = ANY()`.

#### Problem and Solution:
```python
from psqlpy import ConnectionPool

# --- Incorrect ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="SELECT * FROM users WHERE id IN ($1)",
        parameters=[
            (1, 2, 3),
        ],
    )


# --- Correct ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="SELECT * FROM users WHERE id = ANY($1)",
        parameters=[
            (1, 2, 3),
        ],
    )
```

### Wrong binary data

Example error: `binary data has array element type 1043 (character varying) instead of expected 25 (text)`.

This exception tells you that you use wrong data type and you need to specify types explicitly.

For example, when we want to make `WHERE` clause with `ANY` and string values, we need to use `TextArray`, see example below:

#### Problem and Solution:
```python
from psqlpy import ConnectionPool
from psqlpy.extra_types import TextArray

# --- Incorrect ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="SELECT * FROM users WHERE name = ANY($1)",
        parameters=[
            ["Foo", "Bar", "Cafe"],
        ],
    )


# --- Correct ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="SELECT * FROM users WHERE name = ANY($1)",
        parameters=[
            TextArray(["Foo", "Bar", "Cafe"]),
        ],
    )
```

### Cannot insert empty ARRAY

To insert empty array use explicit [Array Type](./usage/types/array_types.md).

#### Problem and Solution:
Let's assume that we have table `arr_table` with field `some_array` of `VARCHAR ARRAY` type.
The main problem that we cannot determine the type of the empty sequence passed from Python side.
```python
from psqlpy import ConnectionPool
from psqlpy.extra_types import VarCharArray

# --- Incorrect ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="INSERT INTO arr_table (some_array) VALUES ($1)",
        parameters=[
            [],
        ],
    )


# --- Correct ---
async def main() -> None:
    pool = ConnectionPool()
    await pool.execute(
        querystring="INSERT INTO arr_table (some_array) VALUES ($1)",
        parameters=[
            VarCharArray([]),
        ],
    )
```
