import typing
from io import BytesIO

import pytest
import pyarrow as pa
from pgpq import ArrowToPostgresBinaryEncoder
from psqlpy import ConnectionPool

pytestmark = pytest.mark.anyio

QUOTED_COLUMN: typing.Final = "a-b"


async def test_binary_copy_to_table_quotes_column_names(
    psql_pool: ConnectionPool,
) -> None:
    """Column names that require quoting work without manual escaping."""
    table_name: typing.Final = "binary_copy_quoted_cols"
    connection = await psql_pool.connection()
    await connection.execute(f'DROP TABLE IF EXISTS "{table_name}"')
    await connection.execute(
        f"""
        CREATE TABLE "{table_name}" (
            "{QUOTED_COLUMN}" TEXT
        )
        """,
    )

    arrow_table = pa.table({QUOTED_COLUMN: ["hello"]})
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
        columns=[QUOTED_COLUMN],
    )

    assert inserted_rows == 1

    result = await connection.execute(
        f'SELECT "{QUOTED_COLUMN}" FROM "{table_name}"',
    )
    assert result.result()[0][QUOTED_COLUMN] == "hello"
