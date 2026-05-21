---
title: COPY FROM STDIN
---

PSQLPy exposes two methods for bulk-loading data via PostgreSQL's `COPY FROM STDIN` protocol.
Both are available on `Connection` and `Transaction`.

## Binary Copy To Table

#### Parameters:

- `source`: bytes, bytearray, or `BytesIO` containing a PostgreSQL binary COPY stream.
- `table_name`: name of the target table.
- `columns`: sequence of column names to load into. When `None`, all table columns are used in their declared order.
- `schema_name`: optional schema for `table_name`.

Stream a pre-encoded PostgreSQL binary COPY payload directly into a table.
Executes `COPY table_name (<columns>) FROM STDIN (FORMAT binary)`.

::: warning
You are responsible for encoding the bytes correctly. Passing an invalid binary COPY stream will result in a database error.
:::

::: tabs

@tab Connection
```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    with open("data.bin", "rb") as f:
        inserted = await connection.binary_copy_to_table(
            source=f.read(),
            table_name="users",
            columns=["id", "username"],
        )
    print(f"Inserted {inserted} rows")
```

@tab Transaction
```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        with open("data.bin", "rb") as f:
            inserted = await transaction.binary_copy_to_table(
                source=f.read(),
                table_name="users",
                columns=["id", "username"],
            )
        print(f"Inserted {inserted} rows")
```

:::

## Copy Records To Table

#### Parameters:

- `table_name`: name of the target table.
- `records`: iterable of records, where each record is a sequence of column values.
- `columns`: sequence of column names to load into. When `None`, all table columns are used in their declared order.
- `schema_name`: optional schema for `table_name`.

Bulk-load plain Python records into a table via the binary `COPY FROM STDIN` protocol.
Column types are introspected from the target table automatically, so each record may contain ordinary Python values — the same types accepted by `execute()`.
Returns the number of inserted rows.

This is the ergonomic alternative to `binary_copy_to_table` when you have Python data rather than a pre-encoded binary stream.

::: tabs

@tab Connection
```python
from datetime import datetime, timezone

async def main() -> None:
    ...
    connection = await db_pool.connection()
    records = [
        (1, "alpha", 1.5,  datetime(2026, 1, 1, tzinfo=timezone.utc)),
        (2, "beta",  2.25, datetime(2026, 1, 2, tzinfo=timezone.utc)),
        (3, "gamma", None, datetime(2026, 1, 3, tzinfo=timezone.utc)),
    ]
    inserted = await connection.copy_records_to_table(
        table_name="measurements",
        records=records,
    )
    print(f"Inserted {inserted} rows")
```

@tab Transaction
```python
from datetime import datetime, timezone

async def main() -> None:
    ...
    connection = await db_pool.connection()
    records = [
        (1, "alpha", 1.5,  datetime(2026, 1, 1, tzinfo=timezone.utc)),
        (2, "beta",  2.25, datetime(2026, 1, 2, tzinfo=timezone.utc)),
        (3, "gamma", None, datetime(2026, 1, 3, tzinfo=timezone.utc)),
    ]
    async with connection.transaction() as transaction:
        inserted = await transaction.copy_records_to_table(
            table_name="measurements",
            records=records,
        )
    print(f"Inserted {inserted} rows")
```

:::

You can load only a subset of columns by providing the `columns` argument:

::: tabs

@tab Connection
```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    inserted = await connection.copy_records_to_table(
        table_name="measurements",
        records=[(1, "alpha"), (2, "beta")],
        columns=["id", "label"],
    )
```

@tab Transaction
```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
    async with connection.transaction() as transaction:
        inserted = await transaction.copy_records_to_table(
            table_name="measurements",
            records=[(1, "alpha"), (2, "beta")],
            columns=["id", "label"],
        )
```

:::
