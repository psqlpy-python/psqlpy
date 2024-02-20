# PSQLPy - Async PostgreSQL driver for Python written in Rust.

Driver for PostgreSQL written fully in Rust and exposed to Python.  
*Normal documentation is in development.*

## Installation

You can install package with `pip` or `poetry`.

poetry:
```bash
> poetry add psqlpy
```
pip:
```bash
> pip install psqlpy
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

from psqlpy import PSQLPool


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
Please take into account that each new execute gets new connection from connection pool. 

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

## Connection
You can work with connection instead of DatabasePool.
```python
from typing import Any

from psqlpy import PSQLPool


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

    connection = await db_pool.connection()

    res: list[dict[str, Any]] = await connection.execute(
        "SELECT * FROM users",
    )

    print(res)
    # You don't need to close connection by yourself,
    # rust does it instead.
```

## Transactions
Of course it's possible to use transactions with this driver.  
It's as easy as possible and sometimes it copies common functionality from PsycoPG and AsyncPG.

### Transaction parameters
In process of transaction creation it is possible to specify some arguments to configure transaction.

- `isolation_level`: level of the isolation. By default - `None`.
- `read_variant`: read option. By default - `None`.

### You can use transactions as async context managers
By default async context manager only begins and commits transaction automatically.
```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
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

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
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

### Transactions can be rolled back
You must understand that rollback can be executed only once per transaction.  
After it's execution transaction state changes to `done`.  
If you want to use `ROLLBACK TO SAVEPOINT`, see below.
```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
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

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
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

    await transaction.commit()
```

### Transaction RELEASE SAVEPOINT
It's possible to release savepoint

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    # Create new savepoint
    await transaction.savepoint("test_savepoint")
    # Release savepoint
    await transaction.release_savepoint("test_savepoint")

    await transaction.commit()
```

## Cursors
Library supports PostgreSQL cursors.

Cursors can be created only in transaction. In addition, cursor supports async iteration.

### Cursor parameters
In process of cursor creation you can specify some configuration parameters.
- `querystring`: query for the cursor. Required.
- `parameters`: parameters for the query. Not Required.
- `fetch_number`: number of records per fetch if cursor is used as an async iterator. If you are using `.fetch()` method you can pass different fetch number. Not required. Default - 10.
- `scroll`: set `SCROLL` if True or `NO SCROLL` if False. Not required. By default - `None`.

```python
from typing import Any

from psqlpy import PSQLPool, IsolationLevel


db_pool = PSQLPool()

async def main() -> None:
    await db_pool.startup()

    connection = await db_pool.connection()
    transaction = connection.transaction(
        isolation_level=IsolationLevel.Serializable,
    )

    await transaction.begin()
    # Create new savepoint
    cursor = await transaction.cursor(
        querystring="SELECT * FROM users WHERE username = $1",
        parameters=["SomeUserName"],
        fetch_number=100,
    )

    # You can manually fetch results from cursor
    results: list[dict[str, Any]] = await cursor.fetch(fetch_number=8)

    # Or you can use it as an async iterator.
    async for fetched_result in cursor:
        print(fetched_result.result())
    
    # If you want to close cursor, please do it manually.
    await cursor.close()
    
    await transaction.commit()
```

### Cursor operations
Available cursor operations:
- FETCH count - `cursor.fetch(fetch_number=)`
- FETCH NEXT - `cursor.fetch_next()`
- FETCH PRIOR - `cursor.fetch_prior()`
- FETCH FIRST - `cursor.fetch_first()`
- FETCH LAST - `cursor.fetch_last()`
- FETCH ABSOLUTE - `cursor.fetch_absolute(absolute_number=)`
- FETCH RELATIVE - `cursor.fetch_relative(relative_number=)`
- FETCH FORWARD ALL - `cursor.fetch_forward_all()`
- FETCH BACKWARD backward_count - `cursor.fetch_backward(backward_count=)`
- FETCH BACKWARD ALL - `cursor.fetch_backward_all()`

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

import uuid

from psqlpy import PSQLPool

from psqlpy.extra_types import (
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