from __future__ import annotations

import typing

if typing.TYPE_CHECKING:
    from psqlpy import Connection, Transaction


async def count_rows_in_test_table(
    table_name: str,
    transaction: Transaction | Connection,
) -> int:
    query_result: typing.Final = await transaction.execute(
        f"SELECT COUNT(*) FROM {table_name}",
    )
    return query_result.result()[0]["count"]
