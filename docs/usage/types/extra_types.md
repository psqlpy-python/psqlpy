---
title: Extra Types
---

PSQLPy has additional types due to the inability to accurately recognize the type passed from Python.

All extra types available from Python with mapping to PostgreSQL type and Rust type.
| PSQLPy type | PostgreSQL type | Rust Type |
| :---: | :---: | :---: |
| BigInt | BigInt | i64 |
| Integer | Integer | i32 |
| SmallInt | SmallInt | i16 |
| Float32 | FLOAT4 | f32 |
| Float64 | FLOAT8 | f64 |
| PyVarChar | VarChar | String |
| PyText | Text | String |
| PyJSON | JSON | serde::Value |
| PyJSONB | JSONB | serde::Value |
| PyMacAddr6 | MacAddr | MacAddr6 |
| PyMacAddr8 | MacAddr8 | MacAddr8 |

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

    await db_pool.execute(
        "INSERT INTO numbers (index, elf_life, elon_musk_money) VALUES ($1, $2, $3, $4, $5)",
        [SmallInt(101), Integer(10500), BigInt(300000000000), Float32(123.11), Float64(222.12)],
    )
    db_pool.close()
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

    await db_pool.execute(
        "INSERT INTO banners (title, description) VALUES ($1, $2)",
        ["SomeTitle", PyText("Very long description")],
    )
    # Alternatively, you can do this:
    await db_pool.execute(
        "INSERT INTO banners (title, description) VALUES ($1, $2)",
        [PyVarChar("SomeTitle"), PyText("Very long description")],
    )
    db_pool.close()
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

    await db_pool.execute(
        "INSERT INTO users (additional_user_info) VALUES ($1)",
        [PyJSONB(list_for_jsonb_field)],
    )
    await db_pool.execute(
        "INSERT INTO users (additional_user_info) VALUES ($1)",
        [dict_for_jsonb_field,],
    )

    db_pool.close()
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

    await db_pool.execute(
        "INSERT INTO devices (device_macaddr6, device_macaddr8) VALUES ($1, $2)",
        [
            PyMacAddr6("08:00:2b:01:02:03"),
            PyMacAddr8("08:00:2b:01:02:03:04:05"),
        ],
    )

    db_pool.close()
```
