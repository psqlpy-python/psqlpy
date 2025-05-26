from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from psqlpy import ConnectionPool, Cursor

pytestmark = pytest.mark.anyio


async def test_cursor_fetchmany(
    number_database_records: int,
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch with custom number of fetch."""
    result = await test_cursor.fetchmany(size=number_database_records // 2)
    assert len(result.result()) == number_database_records // 2


async def test_cursor_fetchone(
    test_cursor: Cursor,
) -> None:
    result = await test_cursor.fetchone()
    assert len(result.result()) == 1


async def test_cursor_fetchall(
    number_database_records: int,
    test_cursor: Cursor,
) -> None:
    result = await test_cursor.fetchall()
    assert len(result.result()) == number_database_records


async def test_cursor_start(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    connection = await psql_pool.connection()
    cursor = connection.cursor(
        querystring=f"SELECT * FROM {table_name}",
    )
    await cursor.start()
    results = await cursor.fetchall()

    assert len(results.result()) == number_database_records

    cursor.close()


async def test_cursor_as_async_context_manager(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    connection = await psql_pool.connection()
    async with connection.cursor(
        querystring=f"SELECT * FROM {table_name}",
    ) as cursor:
        results = await cursor.fetchall()

    assert len(results.result()) == number_database_records


async def test_cursor_send_underlying_connection_to_pool(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Test send underlying connection to the pool."""
    async with psql_pool.acquire() as connection:
        async with connection.transaction() as transaction:
            async with transaction.cursor(
                querystring=f"SELECT * FROM {table_name}",
            ) as cursor:
                await cursor.fetchmany(10)
                assert not psql_pool.status().available
            assert not psql_pool.status().available
        assert not psql_pool.status().available
    assert psql_pool.status().available == 1


async def test_cursor_send_underlying_connection_to_pool_manually(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Test send underlying connection to the pool."""
    async with psql_pool.acquire() as connection:
        async with connection.transaction() as transaction:
            cursor = transaction.cursor(querystring=f"SELECT * FROM {table_name}")
            await cursor.start()
            await cursor.fetchmany(10)
            assert not psql_pool.status().available
            cursor.close()
            assert not psql_pool.status().available
        assert not psql_pool.status().available
    assert psql_pool.status().available == 1
