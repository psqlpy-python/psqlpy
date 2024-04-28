from __future__ import annotations

import typing

import pytest
from tests.helpers import count_rows_in_test_table

from psqlpy import ConnectionPool, QueryResult, Transaction, connect
from psqlpy.exceptions import RustPSQLDriverPyBaseError, TransactionError

pytestmark = pytest.mark.anyio


async def test_connection_execute(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test that single connection can execute queries."""
    connection = await psql_pool.connection()

    conn_result = await connection.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    assert isinstance(conn_result, QueryResult)
    assert len(conn_result.result()) == number_database_records


async def test_connection_connection(
    psql_pool: ConnectionPool,
) -> None:
    """Test that connection can create transactions."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()

    assert isinstance(transaction, Transaction)


@pytest.mark.parametrize(
    ("insert_values"),
    [
        [[1, "name1"], [2, "name2"]],
        [[10, "name1"], [20, "name2"], [30, "name3"]],
        [[1, "name1"]],
        [],
    ],
)
async def test_connection_execute_many(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
    insert_values: list[list[typing.Any]],
) -> None:
    connection = await psql_pool.connection()
    try:
        await connection.execute_many(
            f"INSERT INTO {table_name} VALUES ($1, $2)",
            insert_values,
        )
    except TransactionError:
        assert not insert_values
    else:
        assert await count_rows_in_test_table(
            table_name,
            connection,
        ) - number_database_records == len(insert_values)


async def test_connection_fetch_row(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    database_single_query_result: typing.Final = await connection.fetch_row(
        f"SELECT * FROM  {table_name} LIMIT 1",
        [],
    )
    result = database_single_query_result.result()
    assert isinstance(result, dict)


async def test_connection_fetch_row_more_than_one_row(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    with pytest.raises(RustPSQLDriverPyBaseError):
        await connection.fetch_row(
            f"SELECT * FROM  {table_name}",
            [],
        )


async def test_connection_fetch_val(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    value: typing.Final = await connection.fetch_val(
        f"SELECT COUNT(*) FROM {table_name}",
        [],
    )
    assert isinstance(value, int)


async def test_connection_fetch_val_more_than_one_row(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    connection = await psql_pool.connection()
    with pytest.raises(RustPSQLDriverPyBaseError):
        await connection.fetch_row(
            f"SELECT * FROM  {table_name}",
            [],
        )


async def test_connect_method() -> None:
    connection = await connect(
        dsn="postgres://postgres:postgres@localhost:5432/psqlpy_test",
    )

    res = await connection.execute("SELECT 1")
    assert res.result()
