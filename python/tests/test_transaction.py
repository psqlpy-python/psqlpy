import pytest

from psqlpy import PSQLPool
from psqlpy.exceptions import DBTransactionError


@pytest.mark.anyio
async def test_transaction_begin(
    psql_pool: PSQLPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test that transaction must be started with `begin()` method."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()

    with pytest.raises(expected_exception=DBTransactionError):
        await transaction.execute(
            f"SELECT * FROM {table_name}",
        )

    await transaction.begin()

    result = await transaction.execute(
        f"SELECT * FROM {table_name}",
    )

    assert len(result.result()) == number_database_records


@pytest.mark.anyio
async def test_transaction_commit(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    """Test that transaction commit command."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()

    test_name: str = "test_name"
    await transaction.execute(
        f"INSERT INTO {table_name} VALUES ($1, $2)",
        parameters=[100, test_name],
    )

    # Make request from other connection, it mustn't know
    # about new INSERT data before commit.
    result = await psql_pool.execute(
        f"SELECT * FROM {table_name} WHERE name = $1",
        parameters=[test_name],
    )
    assert not result.result()

    await transaction.commit()

    result = await psql_pool.execute(
        f"SELECT * FROM {table_name} WHERE name = $1",
        parameters=[test_name],
    )

    assert len(result.result())


@pytest.mark.anyio
async def test_transaction_savepoint(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    """Test that it's possible to rollback to savepoint."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()

    test_name = "test_name"
    savepoint_name = "sp1"
    await transaction.savepoint(savepoint_name=savepoint_name)
    await transaction.execute(
        f"INSERT INTO {table_name} VALUES ($1, $2)",
        parameters=[100, test_name],
    )
    result = await transaction.execute(
        f"SELECT * FROM {table_name} WHERE name = $1",
        parameters=[test_name],
    )
    assert result.result()

    await transaction.rollback_to(savepoint_name=savepoint_name)
    result = await psql_pool.execute(
        f"SELECT * FROM {table_name} WHERE name = $1",
        parameters=[test_name],
    )
    assert not len(result.result())

    await transaction.commit()
