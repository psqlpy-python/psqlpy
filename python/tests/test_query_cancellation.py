"""A query whose awaiting task is cancelled must not keep running on the server.

Each test makes another session lock row 1 of the default test table, so a
statement on that row waits on the server until the test gives up on it.
"""

import asyncio
from collections.abc import AsyncGenerator

import pytest
from psqlpy import ConnectionPool
from psqlpy.exceptions import ConnectionExecuteError

from tests.helpers import (
    SERVER_WAIT,
    abandon,
    eventually,
    is_not_running,
    is_running,
    row_exists,
    row_locked,
)

pytestmark = pytest.mark.anyio


@pytest.fixture
async def small_pool(dsn: str) -> AsyncGenerator[ConnectionPool, None]:
    """Pool under test; the tests watch it through `psql_pool`."""
    pool = ConnectionPool(dsn=dsn, max_db_pool_size=2)
    yield pool
    pool.close()


def delete_row_1(table_name: str) -> str:
    return f"DELETE FROM {table_name} WHERE id = 1"


async def test_timed_out_query_is_cancelled_on_server(
    psql_pool: ConnectionPool,
    small_pool: ConnectionPool,
    table_name: str,
) -> None:
    """An asyncio.wait_for timeout stops the query on the server, so nothing commits."""
    query = delete_row_1(table_name)

    async def delete() -> None:
        async with small_pool.acquire() as connection:
            await connection.execute(query)

    async with row_locked(psql_pool, table_name):
        waiter = asyncio.create_task(asyncio.wait_for(delete(), 2))
        assert await eventually(lambda: is_running(psql_pool, query))
        with pytest.raises(asyncio.TimeoutError):
            await waiter
        assert await eventually(lambda: is_not_running(psql_pool, query))

    assert await row_exists(psql_pool, table_name, 1)


async def test_cancelled_query_in_transaction_is_cancelled_on_server(
    psql_pool: ConnectionPool,
    small_pool: ConnectionPool,
    table_name: str,
) -> None:
    query = delete_row_1(table_name)
    connection = await small_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()
    try:
        async with row_locked(psql_pool, table_name):
            await abandon(psql_pool, query, transaction.execute(query))
            assert await eventually(lambda: is_not_running(psql_pool, query))
    finally:
        await transaction.rollback()
        connection.close()

    assert await row_exists(psql_pool, table_name, 1)


async def test_cancelled_execute_many_is_cancelled_on_server(
    psql_pool: ConnectionPool,
    small_pool: ConnectionPool,
    table_name: str,
) -> None:
    """The statements of an abandoned batch are rolled back, not committed later."""
    query = f"DELETE FROM {table_name} WHERE id = $1"

    async def delete_rows() -> None:
        async with small_pool.acquire() as connection:
            await connection.execute_many(query, [[1], [2]])

    async with row_locked(psql_pool, table_name):
        await abandon(psql_pool, query, delete_rows())
        assert await eventually(lambda: is_not_running(psql_pool, query))

    assert await row_exists(psql_pool, table_name, 1)
    assert await row_exists(psql_pool, table_name, 2)


async def test_connection_with_cancelled_query_is_not_reused(
    psql_pool: ConnectionPool,
    small_pool: ConnectionPool,
    table_name: str,
) -> None:
    """The next acquire gets another backend, even while the row is still locked."""
    query = delete_row_1(table_name)
    async with small_pool.acquire() as connection:
        first_pid = await connection.fetch_val("SELECT pg_backend_pid()")

    async def delete() -> None:
        async with small_pool.acquire() as connection:
            await connection.execute(query)

    async with row_locked(psql_pool, table_name):
        await abandon(psql_pool, query, delete())
        async with small_pool.acquire() as connection:
            next_pid = await asyncio.wait_for(
                connection.fetch_val("SELECT pg_backend_pid()"),
                SERVER_WAIT,
            )

    assert next_pid != first_pid


async def test_connection_leaves_pool_after_cancelled_query(
    psql_pool: ConnectionPool,
    small_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Later queries on the connection run, and closing it takes it out of the pool."""
    query = delete_row_1(table_name)
    connection = await small_pool.connection()
    async with row_locked(psql_pool, table_name):
        await abandon(psql_pool, query, connection.execute(query))
        assert await asyncio.wait_for(connection.fetch_val("SELECT 1"), SERVER_WAIT) == 1

    size_before_close = small_pool.status().size
    connection.close()

    assert small_pool.status().size == size_before_close - 1


async def test_completed_query_keeps_its_connection(small_pool: ConnectionPool) -> None:
    async with small_pool.acquire() as connection:
        first_pid = await connection.fetch_val("SELECT pg_backend_pid()")
    async with small_pool.acquire() as connection:
        second_pid = await connection.fetch_val("SELECT pg_backend_pid()")

    assert second_pid == first_pid
    assert small_pool.status().size == 1


async def test_failed_pipeline_keeps_its_connection(small_pool: ConnectionPool) -> None:
    """A failing statement does not abandon the rest of its pipeline."""
    async with small_pool.acquire() as connection:
        first_pid = await connection.fetch_val("SELECT pg_backend_pid()")
        transaction = connection.transaction()
        await transaction.begin()
        with pytest.raises(ConnectionExecuteError):
            await transaction.pipeline([("SELECT 1 / 0", []), ("SELECT 1", [])])
        await transaction.rollback()
    async with small_pool.acquire() as connection:
        second_pid = await connection.fetch_val("SELECT pg_backend_pid()")

    assert second_pid == first_pid
