import pytest

from psqlpy import PSQLPool, QueryResult, Transaction

pytestmark = pytest.mark.anyio


async def test_connection_execute(
    psql_pool: PSQLPool,
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


async def test_connection_transaction(
    psql_pool: PSQLPool,
) -> None:
    """Test that connection can create transactions."""
    connection = await psql_pool.connection()
    transaction = connection.transaction()

    assert isinstance(transaction, Transaction)
