import pytest
from tests.conftest import DefaultPydanticModel, DefaultPythonModelClass

from psqlpy import PSQLPool

pytestmark = pytest.mark.anyio


async def test_as_class(
    psql_pool: PSQLPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test `as_class()` method."""
    select_result = await psql_pool.execute(
        f"SELECT * FROM {table_name}",
    )

    as_pydantic = select_result.as_class(
        as_class=DefaultPydanticModel,
    )
    assert len(as_pydantic) == number_database_records

    for single_record in as_pydantic:
        assert isinstance(single_record, DefaultPydanticModel)

    as_py_class = select_result.as_class(
        as_class=DefaultPythonModelClass,
    )

    assert len(as_py_class) == number_database_records

    for single_py_record in as_py_class:
        assert isinstance(single_py_record, DefaultPythonModelClass)
