import pytest

from psqlpy import PSQLPool


@pytest.mark.anyio
async def test_cursor_fetch(
    psql_pool: PSQLPool,
    table_name: str,
    number_database_records: int,
) -> None:
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()
    await transaction.cursor(
        querystring=f"SELECT * FROM {table_name}",
    )

    await transaction.commit()
