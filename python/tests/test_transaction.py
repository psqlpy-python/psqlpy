from __future__ import annotations

import pytest

from psqlpy import Cursor, IsolationLevel, PSQLPool, ReadVariant
from psqlpy.exceptions import DBTransactionError, RustPSQLDriverPyBaseError


@pytest.mark.anyio()
@pytest.mark.parametrize(
    ("isolation_level", "deferrable", "read_variant"),
    [
        (None, None, None),
        (IsolationLevel.ReadCommitted, True, ReadVariant.ReadOnly),
        (IsolationLevel.ReadUncommitted, False, ReadVariant.ReadWrite),
        (IsolationLevel.RepeatableRead, True, ReadVariant.ReadOnly),
        (IsolationLevel.Serializable, False, ReadVariant.ReadWrite),
    ],
)
async def test_transaction_init_parameters(
    psql_pool: PSQLPool,
    table_name: str,
    isolation_level: IsolationLevel | None,
    deferrable: bool | None,
    read_variant: ReadVariant | None,
) -> None:
    connection = await psql_pool.connection()
    async with connection.transaction(
        isolation_level=isolation_level,
        deferrable=deferrable,
        read_variant=read_variant,
    ) as transaction:
        await transaction.execute("SELECT 1")
        try:
            await transaction.execute(
                f"INSERT INTO {table_name} VALUES ($1, $2)",
                parameters=[100, "test_name"],
            )
        except RustPSQLDriverPyBaseError:
            assert read_variant is ReadVariant.ReadOnly
        else:
            assert read_variant is not ReadVariant.ReadOnly


@pytest.mark.anyio()
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


@pytest.mark.anyio()
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


@pytest.mark.anyio()
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


@pytest.mark.anyio()
async def test_transaction_rollback(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    """Test that ROLLBACK works correctly."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()

    test_name = "test_name"
    await transaction.execute(
        f"INSERT INTO {table_name} VALUES ($1, $2)",
        parameters=[100, test_name],
    )

    result = await transaction.execute(
        f"SELECT * FROM {table_name} WHERE name = $1",
        parameters=[test_name],
    )
    assert result.result()

    await transaction.rollback()

    with pytest.raises(expected_exception=DBTransactionError):
        await transaction.execute(
            f"SELECT * FROM {table_name} WHERE name = $1",
            parameters=[test_name],
        )

    result_from_conn = await psql_pool.execute(
        f"INSERT INTO {table_name} VALUES ($1, $2)",
        parameters=[100, test_name],
    )

    assert not (result_from_conn.result())


@pytest.mark.anyio()
async def test_transaction_release_savepoint(
    psql_pool: PSQLPool,
) -> None:
    """Test that it is possible to acquire and release savepoint."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()

    sp_name_1 = "sp1"
    sp_name_2 = "sp2"

    await transaction.savepoint(sp_name_1)

    with pytest.raises(expected_exception=DBTransactionError):
        await transaction.savepoint(sp_name_1)

    await transaction.savepoint(sp_name_2)

    await transaction.release_savepoint(sp_name_1)
    await transaction.savepoint(sp_name_1)


@pytest.mark.anyio()
async def test_transaction_cursor(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    """Test that transaction can create cursor."""
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        cursor = await transaction.cursor(f"SELECT * FROM {table_name}")

        assert isinstance(cursor, Cursor)
