---
title: Supported Types
---

## Simple Type
Here you can find all types supported by `PSQLPy`. If PSQLPy isn't `-`, you can go to the `Extra Types` for more information.

| Python type | PSQLPy extra type | PostgreSQL Type
| :---: | :---: | :---: |
| None | - | NULL |
| bool | - | BOOL |
| bytes | - | BYTEA |
| str | - | VARCHAR |
| str | PyVarChar | VARCHAR |
| str | PyText | TEXT |
| str | - | XML |
| int | SmallInt | SMALLINT |
| int | INTEGER | INTEGER |
| int | - | INTEGER |
| int | BIGINT | BIGINT |
| float | - | FLOAT4 |
| float | Float32 | FLOAT4 |
| float | Float64 | FLOAT8 |
| datetime.date | - | DATE |
| datetime.time | - | TIME |
| datetime.datetime | - | TIMESTAMP |
| datetime.datetime | - | TIMESTAMPTZ |
| UUID | - | UUID |
| dict | - | JSONB |
| dict | PyJSONB | JSONB |
| dict | PyJSON | JSON |
| Mac Address 6 | PyMacAddr6 | MacAddr |
| Mac Address 8 | PyMacAddr8 | MacAddr |
| IPv4Address | - | INET |
| IPv6Address | - | INET |
| decimal.Decimal | - | NUMERIC |
| int/str | Money | MONEY |

::: important
DECIMAL PostgreSQL type isn't supported, use NUMERIC instead.
:::


## Array Type

You can make arrays with any type of `Simple Type`s.

#### Example:
```sql
CREATE TABLE test (arr_of_json JSONB ARRAY)
```

## Composite Type
`PSQLPy` supports composite types.

You can create your own types in PostgreSQL, we will return you `dict`.
```sql
CREATE TYPE custom_type AS (name VARCHAR, metadata JSONB);
CREATE TABLE custom_table (user_info custom_type);
```

Let's insert some data.

```sql
INSERT INTO custom_table VALUES (ROW('Alex', '{"age": 50}'));
```

Now we can see what result will be returned.
```python
from typing import Final

from psqlpy import ConnectionPool, QueryResult
from psqlpy.extra_types import SmallInt, Integer, BigInt


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    result = await db_pool.execute(
        "SELECT user_info FROM custom_table",
    )
    print(result.result()[0])
```
It will return:
```json
[
    {
        "user_info": {
            "name": "Alex",
            "metadata": {
                "age": 50,
            },
        },
    }
]
```

## Enum Type
You can use ENUM type in `PostgreSQL` and `Python`.

Let's assume we create `Enum` `Weather` and table with it.
```sql
CREATE TYPE weather AS ENUM ('sun', 'not sun')
CREATE table weather_plus (is_weather_good weather)
```

Let's see how we can INSERT and SELECT such data.

```python
from enum import Enum
from typing import Final

from psqlpy import ConnectionPool, QueryResult


class Weather(str, Enum):
    SUN = "sun"
    NOT_SUN = "not sun"


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    # Insert new data
    await db_pool.execute(
        querystring="INSERT INTO weather_plus VALUES($1)",
        parameters=[Weather.SUN],
    )

    # Or you can pass string directly
    await db_pool.execute(
        querystring="INSERT INTO weather_plus VALUES($1)",
        parameters=["sun"],
    )

    result = await db_pool.execute(
        querystring="SELECT * FROM weather_plus",
    )
    print(result.result()[0])
```
You will receive:
```json
[
    {
        "is_weather_good": "sun",
    },
]
```
