from __future__ import annotations

import math

import pytest

from psqlpy import ConnectionPool, Cursor, QueryResult, Transaction

pytestmark = pytest.mark.anyio


async def test_cursor_fetch(
    number_database_records: int,
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch with custom number of fetch."""
    result = await test_cursor.fetch(fetch_number=number_database_records // 2)
    assert len(result.result()) == number_database_records // 2


async def test_cursor_fetch_next(
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch next."""
    result = await test_cursor.fetch_next()
    assert len(result.result()) == 1


async def test_cursor_fetch_prior(
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch prior."""
    result = await test_cursor.fetch_prior()
    assert len(result.result()) == 0

    await test_cursor.fetch(fetch_number=2)
    result = await test_cursor.fetch_prior()
    assert len(result.result()) == 1


async def test_cursor_fetch_first(
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch first."""
    fetch_first = await test_cursor.fetch(fetch_number=1)

    await test_cursor.fetch(fetch_number=3)

    first = await test_cursor.fetch_first()

    assert fetch_first.result() == first.result()


async def test_cursor_fetch_last(
    test_cursor: Cursor,
    number_database_records: int,
) -> None:
    """Test cursor fetch last."""
    all_res = await test_cursor.fetch(
        fetch_number=number_database_records,
    )

    last_res = await test_cursor.fetch_last()

    assert all_res.result()[-1] == last_res.result()[0]


async def test_cursor_fetch_absolute(
    test_cursor: Cursor,
    number_database_records: int,
) -> None:
    """Test cursor fetch Absolute."""
    all_res = await test_cursor.fetch(
        fetch_number=number_database_records,
    )

    first_record = await test_cursor.fetch_absolute(
        absolute_number=1,
    )
    last_record = await test_cursor.fetch_absolute(
        absolute_number=-1,
    )

    assert all_res.result()[0] == first_record.result()[0]
    assert all_res.result()[-1] == last_record.result()[0]


async def test_cursor_fetch_relative(
    test_cursor: Cursor,
    number_database_records: int,
) -> None:
    """Test cursor fetch Relative."""
    first_absolute = await test_cursor.fetch_relative(
        relative_number=1,
    )

    assert first_absolute.result()

    await test_cursor.fetch(
        fetch_number=number_database_records,
    )
    records = await test_cursor.fetch_relative(
        relative_number=1,
    )

    assert not (records.result())


async def test_cursor_fetch_forward_all(
    test_cursor: Cursor,
    number_database_records: int,
) -> None:
    """Test that cursor execute FETCH FORWARD ALL correctly."""
    default_fetch_number = 2
    await test_cursor.fetch(fetch_number=default_fetch_number)

    rest_results = await test_cursor.fetch_forward_all()

    assert len(rest_results.result()) == number_database_records - default_fetch_number


async def test_cursor_fetch_backward(
    test_cursor: Cursor,
) -> None:
    """Test cursor backward fetch."""
    must_be_empty = await test_cursor.fetch_backward(backward_count=10)
    assert not (must_be_empty.result())

    default_fetch_number = 5
    await test_cursor.fetch(fetch_number=default_fetch_number)

    expected_number_of_results = 3
    must_not_be_empty = await test_cursor.fetch_backward(
        backward_count=expected_number_of_results,
    )
    assert len(must_not_be_empty.result()) == expected_number_of_results


async def test_cursor_fetch_backward_all(
    test_cursor: Cursor,
) -> None:
    """Test cursor `fetch_backward_all`."""
    must_be_empty = await test_cursor.fetch_backward_all()
    assert not (must_be_empty.result())

    default_fetch_number = 5
    await test_cursor.fetch(fetch_number=default_fetch_number)

    must_not_be_empty = await test_cursor.fetch_backward_all()
    assert len(must_not_be_empty.result()) == default_fetch_number - 1


async def test_cursor_as_async_manager(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test cursor async manager and async iterator."""
    connection = await psql_pool.connection()
    transaction: Transaction
    cursor: Cursor
    all_results: list[QueryResult] = []
    expected_num_results = math.ceil(number_database_records / 3)
    fetch_number = 3
    async with connection.transaction() as transaction, transaction.cursor(
        querystring=f"SELECT * FROM {table_name}",
        fetch_number=fetch_number,
    ) as cursor:
        async for result in cursor:
            all_results.append(result)

    assert len(all_results) == expected_num_results


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
                await cursor.fetch(10)
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
            await cursor.fetch(10)
            assert not psql_pool.status().available
            await cursor.close()
            assert not psql_pool.status().available
        assert not psql_pool.status().available
    assert psql_pool.status().available == 1
