# Async PostgreSQL driver for Python written in Rust.

Driver for PostgreSQL written fully in Rust and exposed to Python.  
*Normal documentation is in development.*

## Installation

You can install package with `pip` or `poetry`.

poetry:
```bash
> poetry add psql-rust-driver
```
pip:
```bash
> pip install psql-rust-driver
```

Or you can build it by yourself. To do it, install stable rust and [maturin](https://github.com/PyO3/maturin).
```
> maturin develop --release
```

## Usage
Usage is as easy as possible.  
Create new instance of PSQLPool, startup it and start querying.
```python
from typing import Any
import asyncio

from rust_psql_driver import PSQLPool


db_pool = PSQLPool(
    username="postgres",
    password="pg_password",
    host="localhost",
    port=5432,
    db_name="postgres",
    max_db_pool_size=2,
)

async def main() -> None:
    await db_pool.startup()

    res: list[dict[str, Any]] = await db_pool.execute(
        "SELECT * FROM users",
    )

    print(res)
    # You don't need to close Database Pool by yourself,
    # rust does it instead.

```

## Query parameters
You can pass parameters into queries.  
Parameters can be passed in any `execute` method as the second parameter, it must be a list.  
Any placeholder must be marked with `$< num>`.

```python
    res: list[dict[str, Any]] = await db_pool.execute(
        "SELECT * FROM users WHERE user_id = $1 AND first_name = $2",
        [100, "RustDriver"],
    )
```

Support all types from Python is in development, but there are support for all basic types. More you can find here: ... 

## Transactions
Of course it's possible to use transactions with this driver.  
It's as easy as possible and sometimes it copies common functionality from PsycoPG and AsyncPG.

### You can use transactions as async context managers
By default async context manager only begins and commits transaction automatically.
```python
from typing import Any
import asyncio

from rust_psql_driver import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    transaction = await db_pool.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    async with transaction:
        res: list[dict[str, Any]] = await transaction.execute(
            "SELECT * FROM users",
        )

    print(res)
    # You don't need to close Database Pool by yourself,
    # rust does it instead.
```

### Or you can control transaction fully on your own.
```python
from typing import Any
import asyncio

from rust_psql_driver import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    transaction = await db_pool.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    await transaction.execute(
        "INSERT INTO users VALUES ($1)",
        ["Some data"],
    )
    # You must commit the transaction by your own 
    # or your changes will be vanished.
    await transaction.commit()

    print(res)
    # You don't need to close Database Pool by yourself,
    # rust does it instead.
```

### Transactions can be roll backed
You must understand that rollback can be executed only once per transaction.  
After it's execution transaction state changes to `done`.  
If you want to use `ROLLBACK TO SAVEPOINT`, see below.
```python
from typing import Any
import asyncio

from rust_psql_driver import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    transaction = await db_pool.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    await transaction.execute(
        "INSERT INTO users VALUES ($1)",
        ["Some data"],
    )
    await transaction.rollback()
```

### Transaction ROLLBACK TO SAVEPOINT
You can rollback your transaction to the specified savepoint, but before it you must create it.

```python
from typing import Any
import asyncio

from rust_psql_driver import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    transaction = await db_pool.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    # Create new savepoint
    await transaction.savepoint("test_savepoint")

    await transaction.execute(
        "INSERT INTO users VALUES ($1)",
        ["Some data"],
    )
    # Rollback to specified SAVEPOINT.
    await transaction.rollback_to("test_savepoint")
```

## Extra Types
Sometimes it's impossible to identify which type user tries to pass as a argument. But Rust is a strongly typed programming language so we have to help.

| Extra Type in Python  | Type in PostgreSQL | Type in Rust |
| ------------- | ------------- | ------------- 
| SmallInt  | SmallInt  | i16 |
| Integer  | Integer  | i32 |
| BigInt  | BigInt  | i64 |
| PyUUID  | UUID  | Uuid |
| PyJSON  | JSON, JSONB  | Value |

```python
from typing import Any
import asyncio
import uuid

from rust_psql_driver import PSQLPool

from psql_rust_driver.extra_types import (
    SmallInt,
    Integer,
    BigInt,
    PyUUID,
    PyJSON,
)


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    res: list[dict[str, Any]] = await db_pool.execute(
        "INSERT INTO users VALUES ($1, $2, $3, $4, $5)",
        [
            SmallInt(100),
            Integer(10000),
            BigInt(9999999),
            PyUUID(uuid.uuid4().hex),
            PyJSON(
                [
                    {"we": "have"},
                    {"list": "of"},
                    {"dicts": True},
                ],
            )
        ]
    )

    print(res)
    # You don't need to close Database Pool by yourself,
    # rust does it instead.

```