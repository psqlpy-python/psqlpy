import typing
from datetime import datetime, timezone

import pytest
from psqlpy import ConnectionPool
from psqlpy.exceptions import PyToRustValueMappingError

pytestmark = pytest.mark.anyio


async def _setup_target_table(psql_pool: ConnectionPool, name: str) -> None:
    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP TABLE IF EXISTS {name}")
        await connection.execute(
            f"""
            CREATE TABLE {name} (
                id INTEGER,
                label TEXT,
                weight DOUBLE PRECISION,
                created_at TIMESTAMPTZ
            )
            """,
        )


async def _drop_target_table(psql_pool: ConnectionPool, name: str) -> None:
    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP TABLE IF EXISTS {name}")


async def test_copy_records_to_table_on_connection(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_conn"
    await _setup_target_table(psql_pool, target)
    try:
        records = [
            (1, "alpha", 1.5, datetime(2026, 1, 1, tzinfo=timezone.utc)),
            (2, "beta", 2.25, datetime(2026, 1, 2, tzinfo=timezone.utc)),
            (3, "gamma", None, datetime(2026, 1, 3, tzinfo=timezone.utc)),
        ]

        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
            )

        assert inserted == len(records)

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label, weight FROM {target} ORDER BY id",
            )
        rows = result.result()
        assert [(r["id"], r["label"], r["weight"]) for r in rows] == [
            (1, "alpha", 1.5),
            (2, "beta", 2.25),
            (3, "gamma", None),
        ]
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_with_columns_subset(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_subset"
    await _setup_target_table(psql_pool, target)
    try:
        records = [(10, "only-label"), (11, "another")]

        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
                columns=["id", "label"],
            )

        assert inserted == len(records)

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label, weight, created_at FROM {target} ORDER BY id",
            )
        rows = result.result()
        assert [(r["id"], r["label"]) for r in rows] == [
            (10, "only-label"),
            (11, "another"),
        ]
        # Untouched columns must remain NULL
        assert all(r["weight"] is None and r["created_at"] is None for r in rows)
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_in_transaction(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_tx"
    await _setup_target_table(psql_pool, target)
    try:
        records = [(100, "tx-row", 0.0, datetime(2026, 5, 1, tzinfo=timezone.utc))]

        async with (
            psql_pool.acquire() as connection,
            connection.transaction() as tx,
        ):
            inserted = await tx.copy_records_to_table(
                table_name=target,
                records=records,
            )

        assert inserted == 1

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT COUNT(*) AS c FROM {target}",
            )
        assert result.result()[0]["c"] == 1
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_rejects_record_arity_mismatch(
    psql_pool: ConnectionPool,
) -> None:
    target: typing.Final = "copy_records_mismatch"
    await _setup_target_table(psql_pool, target)
    try:
        records = [(1, "missing-rest")]  # table has 4 columns

        async with psql_pool.acquire() as connection:
            with pytest.raises(PyToRustValueMappingError):
                await connection.copy_records_to_table(
                    table_name=target,
                    records=records,
                )
    finally:
        await _drop_target_table(psql_pool, target)


async def test_copy_records_to_table_uses_schema_qualifier(
    psql_pool: ConnectionPool,
) -> None:
    schema: typing.Final = "copy_records_schema"
    target: typing.Final = "tbl"

    async with psql_pool.acquire() as connection:
        await connection.execute(f"DROP SCHEMA IF EXISTS {schema} CASCADE")
        await connection.execute(f"CREATE SCHEMA {schema}")
        await connection.execute(
            f"CREATE TABLE {schema}.{target} (id INTEGER, label TEXT)",
        )

    try:
        records = [(1, "schema-a"), (2, "schema-b")]
        async with psql_pool.acquire() as connection:
            inserted = await connection.copy_records_to_table(
                table_name=target,
                records=records,
                schema_name=schema,
            )

        assert inserted == len(records)

        async with psql_pool.acquire() as connection:
            result = await connection.execute(
                f"SELECT id, label FROM {schema}.{target} ORDER BY id",
            )
        assert [(r["id"], r["label"]) for r in result.result()] == records
    finally:
        async with psql_pool.acquire() as connection:
            await connection.execute(f"DROP SCHEMA IF EXISTS {schema} CASCADE")
