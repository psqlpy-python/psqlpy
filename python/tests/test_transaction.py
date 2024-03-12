from __future__ import annotations

import typing

import pytest
from tests.helpers import count_rows_in_test_table

from psqlpy import Cursor, IsolationLevel, PSQLPool, ReadVariant
from psqlpy.exceptions import DBTransactionError, RustPSQLDriverPyBaseError

pytestmark = pytest.mark.anyio


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


async def test_transaction_cursor(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    """Test that transaction can create cursor."""
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        cursor = await transaction.cursor(f"SELECT * FROM {table_name}")

        assert isinstance(cursor, Cursor)


@pytest.mark.parametrize(
    ("insert_values"),
    [
        [[1, "name1"], [2, "name2"]],
        [[10, "name1"], [20, "name2"], [30, "name3"]],
        [[1, "name1"]],
        [],
    ],
)
async def test_transaction_execute_many(
    psql_pool: PSQLPool,
    table_name: str,
    number_database_records: int,
    insert_values: list[list[typing.Any]],
) -> None:
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        try:
            await transaction.execute_many(
                f"INSERT INTO {table_name} VALUES ($1, $2)",
                insert_values,
            )
        except DBTransactionError:
            assert not insert_values
        else:
            assert await count_rows_in_test_table(
                table_name,
                transaction,
            ) - number_database_records == len(insert_values)


async def test_transaction_fetch_row(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        database_single_query_result: typing.Final = (
            await transaction.fetch_row(
                f"SELECT * FROM  {table_name} LIMIT 1",
                [],
            )
        )
        result = database_single_query_result.result()
        assert isinstance(result, dict)


async def test_transaction_fetch_row_more_than_one_row(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        with pytest.raises(RustPSQLDriverPyBaseError):
            await transaction.fetch_row(
                f"SELECT * FROM  {table_name}",
                [],
            )


async def test_transaction_fetch_val(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        value: typing.Final = await transaction.fetch_val(
            f"SELECT COUNT(*) FROM {table_name}",
            [],
        )
        assert isinstance(value, int)


async def test_transaction_fetch_val_more_than_one_row(
    psql_pool: PSQLPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    async with connection.transaction() as transaction:
        with pytest.raises(RustPSQLDriverPyBaseError):
            await transaction.fetch_row(
                f"SELECT * FROM  {table_name}",
                [],
            )
