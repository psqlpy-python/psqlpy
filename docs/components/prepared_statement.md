---
title: Prepared Statement
---

Representation of PostgreSQL PreparedStatement.

## Usage

::: tabs

@tab Execute
```python
from psqlpy import ConnectionPool, QueryResult

db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)

async def main() -> None:
    connection = await db_pool.connection()
    prepared_stmt = await connection.prepare(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )

    result: QueryResult = await prepared_stmt.execute()
```

@tab Cursor
```python
from psqlpy import ConnectionPool, Cursor, PreparedStatement

db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)

async def main() -> None:
    connection = await db_pool.connection()
    prepared_stmt: PreparedStatement = await connection.prepare(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )

    cursor: Cursor = prepared_stmt.cursor()
```
:::

## PreparedStatement methods

### Execute

Just execute prepared statement.

```python
async def main() -> None:
    connection = await db_pool.connection()
    prepared_stmt = await connection.prepare(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )

    result: QueryResult = await prepared_stmt.execute()
```

### Cursor

Create new Cursor instance from the PreparedStatement.

```python
async def main() -> None:
    connection = await db_pool.connection()
    prepared_stmt = await connection.prepare(
        querystring="SELECT * FROM users WHERE id > $1",
        parameters=[100],
    )

    result: QueryResult = await prepared_stmt.execute()
```