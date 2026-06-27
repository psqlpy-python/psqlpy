"""Tests for the Record pyclass and QueryResult.records() method (AC-5.5)."""

import decimal

import pytest
from psqlpy import ConnectionPool

pytestmark = pytest.mark.anyio


async def test_record_positional_and_named_access(
    psql_pool: ConnectionPool,
) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute(
            "SELECT 1 AS a, 'hello' AS b, 3.14 AS c",
        )

    records = result.records()
    assert len(records) == 1
    row = records[0]

    # positional access
    assert row[0] == 1
    assert row[1] == "hello"

    # negative index resolves to last column (3.14 as numeric)
    assert row[-1] == decimal.Decimal("3.14")

    # by-name access
    assert row["a"] == 1
    assert row["b"] == "hello"


async def test_record_len(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 1 AS x, 2 AS y")
    row = result.records()[0]
    assert len(row) == 2


async def test_record_iteration(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 10 AS x, 20 AS y, 30 AS z")
    row = result.records()[0]
    values = list(row)
    assert values == [10, 20, 30]


async def test_record_get(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 42 AS answer")
    row = result.records()[0]
    assert row.get("answer") == 42
    assert row.get("missing") is None
    assert row.get("missing", 99) == 99


async def test_record_keys(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 1 AS alpha, 2 AS beta")
    row = result.records()[0]
    assert row.keys() == ["alpha", "beta"]


async def test_record_values(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 7 AS p, 8 AS q")
    row = result.records()[0]
    assert row.values() == [7, 8]


async def test_record_items(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 5 AS foo, 'bar' AS baz")
    row = result.records()[0]
    items = row.items()
    assert items == [("foo", 5), ("baz", "bar")]


async def test_record_slice(psql_pool: ConnectionPool) -> None:
    async with psql_pool.acquire() as connection:
        result = await connection.execute(
            "SELECT 1 AS a, 2 AS b, 3 AS c, 4 AS d",
        )
    row = result.records()[0]
    assert row[1:3] == [2, 3]
    assert row[:2] == [1, 2]
    assert row[::2] == [1, 3]


async def test_record_shared_descriptor(psql_pool: ConnectionPool) -> None:
    """All records from one result set share the same column descriptor."""
    async with psql_pool.acquire() as connection:
        result = await connection.execute(
            "SELECT generate_series AS n FROM generate_series(1, 5)",
        )
    records = result.records()
    assert len(records) == 5
    # keys() returns the same column list for all rows
    for row in records:
        assert row.keys() == ["n"]
    # values are distinct per row
    assert [row["n"] for row in records] == list(range(1, 6))


async def test_result_unchanged_by_records(psql_pool: ConnectionPool) -> None:
    """result() still returns dicts after records() is added."""
    async with psql_pool.acquire() as connection:
        result = await connection.execute("SELECT 1 AS n")
    dicts = result.result()
    assert isinstance(dicts[0], dict)
    assert dicts[0]["n"] == 1
