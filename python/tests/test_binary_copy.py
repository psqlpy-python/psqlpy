import os
import typing
from io import BytesIO

import pytest
from pgpq import ArrowToPostgresBinaryEncoder
from psqlpy import ConnectionPool
from pyarrow import parquet

pytestmark = pytest.mark.anyio


async def test_binary_copy_to_table_in_connection(
    psql_pool: ConnectionPool,
) -> None:
    """Test binary copy in connection."""
    table_name: typing.Final = "cars"
    connection = await psql_pool.connection()
    await connection.execute(f"DROP TABLE IF EXISTS {table_name}")
    await connection.execute(
        """
CREATE TABLE IF NOT EXISTS cars (
    model VARCHAR,
    mpg FLOAT8,
    cyl INTEGER,
    disp FLOAT8,
    hp INTEGER,
    drat FLOAT8,
    wt FLOAT8,
    qsec FLOAT8,
    vs INTEGER,
    am INTEGER,
    gear INTEGER,
    carb INTEGER
);
""",
    )

    arrow_table = parquet.read_table(
        f"{os.path.dirname(os.path.abspath(__file__))}/test_data/MTcars.parquet",  # noqa: PTH120, PTH100
    )
    encoder = ArrowToPostgresBinaryEncoder(arrow_table.schema)
    buf = BytesIO()
    buf.write(encoder.write_header())
    for batch in arrow_table.to_batches():
        buf.write(encoder.write_batch(batch))
    buf.write(encoder.finish())
    buf.seek(0)

    inserted_rows = await connection.binary_copy_to_table(
        source=buf,
        table_name=table_name,
    )

    expected_inserted_row: typing.Final = 32

    assert inserted_rows == expected_inserted_row

    real_table_rows: typing.Final = await connection.execute(
        f"SELECT COUNT(*) AS rows_count FROM {table_name}",
    )
    assert real_table_rows.result()[0]["rows_count"] == expected_inserted_row


async def test_binary_copy_to_table_in_transaction(
    psql_pool: ConnectionPool,
) -> None:
    """Test binary copy in transaction."""
    table_name: typing.Final = "cars"

    connection = await psql_pool.connection()
    await connection.execute(f"DROP TABLE IF EXISTS {table_name}")
    await connection.execute(
        """
CREATE TABLE IF NOT EXISTS cars (
    model VARCHAR,
    mpg FLOAT8,
    cyl INTEGER,
    disp FLOAT8,
    hp INTEGER,
    drat FLOAT8,
    wt FLOAT8,
    qsec FLOAT8,
    vs INTEGER,
    am INTEGER,
    gear INTEGER,
    carb INTEGER
);
""",
    )

    arrow_table = parquet.read_table(
        f"{os.path.dirname(os.path.abspath(__file__))}/test_data/MTcars.parquet",  # noqa: PTH120, PTH100
    )
    encoder = ArrowToPostgresBinaryEncoder(arrow_table.schema)
    buf = BytesIO()
    buf.write(encoder.write_header())
    for batch in arrow_table.to_batches():
        buf.write(encoder.write_batch(batch))
    buf.write(encoder.finish())
    buf.seek(0)

    async with psql_pool.acquire() as connection:
        inserted_rows = await connection.binary_copy_to_table(
            source=buf,
            table_name=table_name,
        )

    expected_inserted_row: typing.Final = 32

    assert inserted_rows == expected_inserted_row

    connection = await psql_pool.connection()
    real_table_rows: typing.Final = await connection.execute(
        f"SELECT COUNT(*) AS rows_count FROM {table_name}",
    )
    assert real_table_rows.result()[0]["rows_count"] == expected_inserted_row
