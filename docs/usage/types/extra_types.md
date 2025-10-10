---
title: Extra Types
---

PSQLPy has additional types due to the inability to accurately recognize the type passed from Python.

All extra types available from Python with mapping to PostgreSQL type and Rust type.

::: important
Some of the types are deprecated.

Use standard python types instead of deprecated ones.
:::

| PSQLPy type | PostgreSQL type | Rust Type |
| :---: | :---: | :---: |
| BigInt (Deprecated) | BigInt | i64 |
| Integer (Deprecated) | Integer | i32 |
| SmallInt (Deprecated) | SmallInt | i16 |
| Float32 (Deprecated) | FLOAT4 | f32 |
| Float64 (Deprecated) | FLOAT8 | f64 |
| VarChar (Deprecated) | VarChar | String |
| Text (Deprecated) | Text | String |
| JSON (Deprecated) | JSON | serde::Value |
| JSONB (Deprecated) | JSONB | serde::Value |
| MacAddr6 | MacAddr | MacAddr6 |
| MacAddr8 | MacAddr8 | MacAddr8 |
| Point | Point | Point |
| Box | Rect | Box |
| Path | LineString | Path |
| Line | LineSegment | Line |
| LineSegment | LineSegment | Lseg |
| Circle | Circle | Circle |
| PgVector | Vector | Vector |

::: important
To use `Vector` type in PostgreSQL you need to install it - [pgvector](https://github.com/pgvector/pgvector).
:::


## BigInt & Integer & SmallInt & Float32 & Float64
When integer is passed from Python to Rust, it's impossible to understand what type is required on the Database side.
Because of this restriction if you are trying to insert or update number value, you need to specify type on Python side explicitly.

Let's assume we have table `numbers` in the database:
|  database type | database column name |
| :---: | :---: |
| SmallInt | index |
| Integer | elf_life |
| BigInt | elon_musk_money |
| FLOAT4 | rest_money |
| FLOAT8 | company_money |

And we want to INSERT new data to this table:
```python
from typing import Final

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import SmallInt, Integer, BigInt, Float32, Float64


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()
    async with db_pool.acquire() as connection:
        await connection.execute(
            "INSERT INTO numbers (index, elf_life, elon_musk_money) VALUES ($1, $2, $3, $4, $5)",
            [SmallInt(101), Integer(10500), BigInt(300000000000), Float32(123.11), Float64(222.12)],
        )
```

::: important
These types are limited only by the upper bound.
These classes work not only as wrappers, but also as validators.
For example, you can't pass integer bigger than 32,768 to SmallInt type.
:::

## PyVarChar & PyText
When you need to pass string from Python to PSQLPy and this string must converted into Text PostgreSQL, you need to explicitly mark your string as `PyText`.
If you don't work with PostgreSQL `TEXT` type, you can pass python `str` without any extra type.

Let's assume we have table `banners` in the database:
|  database type | database column name |
| :---: | :---: |
| VarChar | title |
| Text | description |
```python
from typing import Final

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import PyText


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    async with db_pool.acquire() as connection:
        await connection.execute(
            "INSERT INTO banners (title, description) VALUES ($1, $2)",
            ["SomeTitle", PyText("Very long description")],
        )
        # Alternatively, you can do this:
        await connection.execute(
            "INSERT INTO banners (title, description) VALUES ($1, $2)",
            [PyVarChar("SomeTitle"), PyText("Very long description")],
        )
```

## PyJSON & PyJSONB
`PyJSON`/`PyJSONB` type exists only for situations when you want to set list of something to JSON/JSONB field.
If you have default Python dict like above, you DON'T have to use `PyJSON`/`PyJSONB` type.
```python
my_dict = {
    "just": "regular",
    "python": "dictionary",
    "of": [
        "values",
    ],
    "with": {
        "nested": "values",
    }
}
```
On the other side, if you want to set list of values to JSON/JSONB field, you must wrap it in `PyJSON`/`PyJSONB` type, otherwise `PSQLPy` will assume that you passed an array (PostgreSQL `ARRAY`).

Let's assume we have table `users` in the database, and field `additional_user_info` can contain different type of data:
|  database type | database column name |
| :---: | :---: |
| JSONB | additional_user_info |

```python
from typing import Final

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import PyJSON


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    list_for_jsonb_field = [
        {"some": "dict"},
        [
            {"nested": "list of dicts"},
        ],
    ]

    dict_for_jsonb_field = {
        "regular": "dict",
        "with": [
            "list", "of", "values", 100,
        ]
    }

    async with db_pool.acquire() as connection:
        await connection.execute(
            "INSERT INTO users (additional_user_info) VALUES ($1)",
            [PyJSONB(list_for_jsonb_field)],
        )
        await connection.execute(
            "INSERT INTO users (additional_user_info) VALUES ($1)",
            [dict_for_jsonb_field,],
        )
```

## PyMacAddr6 & PyMacAddr8
Mac addresses must be used with `PyMacAddr6` and `PyMacAddr8` types.

Let's assume we have table `devices` in the database:
|  database type | database column name |
| :---: | :---: |
| MACADDR | device_macaddr6 |
| MACADDR8 | device_macaddr8 |

```python
from typing import Final

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import PyMacAddr6, PyMacAddr8


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    async with db_pool.acquire() as connection:
        await connection.execute(
            "INSERT INTO devices (device_macaddr6, device_macaddr8) VALUES ($1, $2)",
            [
                PyMacAddr6("08:00:2b:01:02:03"),
                PyMacAddr8("08:00:2b:01:02:03:04:05"),
            ],
        )
```

## Geo Types
Also in package exists support of PostgreSQL geo types(except Polygon for now).
To use geo types you need specify them directly.

Let's assume we have table `geo_info` with all PostgreSQL geo types in the database:
|  database type | database column name |
| :---: | :---: |
| POINT | map_point |
| BOX | point_district |
| PATH | path_to_point |
| LINE | points_line |
| LSEG | lseg_between_points |
| CIRCLE | point_radius_circle |

```python
from typing import Final

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import Point, Box, Path, Line, LineSegment, Circle


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    async with db_pool.acquire() as connection:
        await connection.execute(
            "INSERT INTO geo_info VALUES ($1, $2, $3, $4, $5, $6)",
            [
                Point([1.5, 2]),
                Box([(1.7, 2.8), (9, 9)]),
                Path([(3.5, 3), (9, 9), (8, 8)]),
                Line([1, -2, 3]),
                LineSegment([(5.6, 3.1), (4, 5)]),
                Circle([5, 1.8, 10]),
            ],
        )
```
