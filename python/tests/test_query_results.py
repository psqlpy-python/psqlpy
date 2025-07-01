from __future__ import annotations

import pytest
from psqlpy import ConnectionPool, QueryResult, SingleQueryResult

pytestmark = pytest.mark.anyio


async def test_result_as_dict(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Test that single connection can execute queries."""
    connection = await psql_pool.connection()

    conn_result = await connection.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    result_list_dicts = conn_result.result()
    single_dict_row = result_list_dicts[0]

    assert isinstance(conn_result, QueryResult)
    assert isinstance(single_dict_row, dict)
    assert single_dict_row.get("id")


async def test_result_as_tuple(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Test that single connection can execute queries."""
    connection = await psql_pool.connection()

    conn_result = await connection.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    result_tuple = conn_result.result(as_tuple=True)
    single_tuple_row = result_tuple[0]

    assert isinstance(conn_result, QueryResult)
    assert isinstance(single_tuple_row, tuple)
    assert single_tuple_row[0] == 1


async def test_single_result_as_dict(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Test that single connection can execute queries."""
    connection = await psql_pool.connection()

    conn_result = await connection.fetch_row(
        querystring=f"SELECT * FROM {table_name} LIMIT 1",
    )
    result_dict = conn_result.result()

    assert isinstance(conn_result, SingleQueryResult)
    assert isinstance(result_dict, dict)
    assert result_dict.get("id")


async def test_single_result_as_tuple(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    """Test that single connection can execute queries."""
    connection = await psql_pool.connection()

    conn_result = await connection.fetch_row(
        querystring=f"SELECT * FROM {table_name} LIMIT 1",
    )
    result_tuple = conn_result.result(as_tuple=True)

    assert isinstance(conn_result, SingleQueryResult)
    assert isinstance(result_tuple, tuple)
    assert result_tuple[0] == 1
