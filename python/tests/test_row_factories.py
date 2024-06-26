from dataclasses import dataclass
from typing import Any, Callable, Dict, Type

import pytest

from psqlpy import ConnectionPool
from psqlpy.row_factories import class_row, tuple_row

pytestmark = pytest.mark.anyio


async def test_tuple_row(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    conn_result = await psql_pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    tuple_res = conn_result.row_factory(row_factory=tuple_row)

    assert len(tuple_res) == number_database_records
    assert isinstance(tuple_res[0], tuple)


async def test_class_row(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    @dataclass
    class ValidationTestModel:
        id: int
        name: str

    conn_result = await psql_pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    class_res = conn_result.row_factory(row_factory=class_row(ValidationTestModel))
    assert len(class_res) == number_database_records
    assert isinstance(class_res[0], ValidationTestModel)


async def test_custom_row_factory(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    @dataclass
    class ValidationTestModel:
        id: int
        name: str

    def to_class(
        class_: Type[ValidationTestModel],
    ) -> Callable[[Dict[str, Any]], ValidationTestModel]:
        def to_class_inner(row: Dict[str, Any]) -> ValidationTestModel:
            return class_(**row)

        return to_class_inner

    conn_result = await psql_pool.execute(
        querystring=f"SELECT * FROM {table_name}",
    )
    class_res = conn_result.row_factory(row_factory=to_class(ValidationTestModel))

    assert len(class_res) == number_database_records
    assert isinstance(class_res[0], ValidationTestModel)
