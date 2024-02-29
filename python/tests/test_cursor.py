import pytest

from psqlpy import Cursor


@pytest.mark.anyio
async def test_cursor_fetch(
    number_database_records: int,
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch with custom number of fetch."""
    result = await test_cursor.fetch(fetch_number=number_database_records // 2)
    assert len(result.result()) == number_database_records // 2


@pytest.mark.anyio
async def test_cursor_fetch_next(
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch next."""
    result = await test_cursor.fetch_next()
    assert len(result.result()) == 1


@pytest.mark.anyio
async def test_cursor_fetch_prior(
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch prior."""
    result = await test_cursor.fetch_prior()
    assert len(result.result()) == 0

    await test_cursor.fetch(fetch_number=2)
    result = await test_cursor.fetch_prior()
    assert len(result.result()) == 1


@pytest.mark.anyio
async def test_cursor_fetch_first(
    test_cursor: Cursor,
) -> None:
    """Test cursor fetch first."""
    fetch_first = await test_cursor.fetch(fetch_number=1)

    await test_cursor.fetch(fetch_number=3)

    first = await test_cursor.fetch_first()

    assert fetch_first.result() == first.result()


@pytest.mark.anyio
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
