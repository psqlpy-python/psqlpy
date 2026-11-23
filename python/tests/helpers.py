from __future__ import annotations

import asyncio
import typing
from contextlib import asynccontextmanager

import pytest

if typing.TYPE_CHECKING:
    from collections.abc import AsyncIterator, Awaitable, Callable

    from psqlpy import Connection, ConnectionPool, Transaction

SERVER_WAIT = 5.0


async def count_rows_in_test_table(
    table_name: str,
    transaction: Transaction | Connection,
) -> int:
    query_result: typing.Final = await transaction.execute(
        f"SELECT COUNT(*) FROM {table_name}",
    )
    return query_result.result()[0]["count"]


@asynccontextmanager
async def row_locked(psql_pool: ConnectionPool, table_name: str) -> AsyncIterator[None]:
    """Hold a lock on row 1 of `table_name` from another session."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()
    await transaction.execute(f"SELECT id FROM {table_name} WHERE id = 1 FOR UPDATE")
    try:
        yield
    finally:
        await transaction.rollback()
        connection.close()


async def eventually(check: Callable[[], Awaitable[bool]]) -> bool:
    loop = asyncio.get_running_loop()
    deadline = loop.time() + SERVER_WAIT
    while loop.time() < deadline:
        if await check():
            return True
        await asyncio.sleep(0.05)
    return False


async def is_running(psql_pool: ConnectionPool, query: str) -> bool:
    async with psql_pool.acquire() as connection:
        rows = await connection.execute(
            "SELECT 1 FROM pg_stat_activity WHERE query = $1 AND state = 'active'",
            [query],
        )
    return bool(rows.result())


async def is_not_running(psql_pool: ConnectionPool, query: str) -> bool:
    return not await is_running(psql_pool, query)


async def abandon(
    psql_pool: ConnectionPool,
    query: str,
    awaitable: Awaitable[object],
) -> None:
    """Cancel the task awaiting `awaitable` once `query` runs on the server."""

    async def wait() -> None:
        await awaitable

    task = asyncio.create_task(wait())
    assert await eventually(lambda: is_running(psql_pool, query))
    task.cancel()
    with pytest.raises(asyncio.CancelledError):
        await task


async def row_exists(psql_pool: ConnectionPool, table_name: str, row_id: int) -> bool:
    async with psql_pool.acquire() as connection:
        rows = await connection.execute(
            f"SELECT 1 FROM {table_name} WHERE id = $1",
            [row_id],
        )
    return bool(rows.result())
