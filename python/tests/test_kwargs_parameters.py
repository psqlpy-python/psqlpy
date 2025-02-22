import pytest
from psqlpy import ConnectionPool
from psqlpy.exceptions import (
    PyToRustValueMappingError,
)

pytestmark = pytest.mark.anyio


async def test_success_default_map_parameters(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    async with psql_pool.acquire() as conn:
        exist_records = await conn.execute(
            f"SELECT * FROM {table_name}",
        )
        result = exist_records.result()

        test_fetch = await conn.execute(
            f"SELECT * FROM {table_name} WHERE id = $(id_)p",
            parameters={
                "id_": result[0]["id"],
            },
        )

    assert test_fetch.result()[0]["id"] == result[0]["id"]
    assert test_fetch.result()[0]["name"] == result[0]["name"]


@pytest.mark.usefixtures("create_table_for_map_parameters_test")
async def test_success_multiple_same_parameters(
    psql_pool: ConnectionPool,
    map_parameters_table_name: str,
) -> None:
    test_name_surname = "Surname"
    test_age = 1
    async with psql_pool.acquire() as conn:
        await conn.execute(
            querystring=(
                f"INSERT INTO {map_parameters_table_name} "
                "(name, surname, age) VALUES ($(name)p, $(surname)p, $(age)p)"
            ),
            parameters={
                "name": test_name_surname,
                "surname": test_name_surname,
                "age": test_age,
            },
        )

        res = await conn.execute(
            querystring=(
                f"SELECT * FROM {map_parameters_table_name} "
                "WHERE name = $(name)p OR surname = $(name)p"
            ),
            parameters={"name": test_name_surname},
        )

    assert res.result()[0]["name"] == test_name_surname
    assert res.result()[0]["surname"] == test_name_surname
    assert res.result()[0]["age"] == test_age


async def test_failed_no_parameter(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    async with psql_pool.acquire() as conn:
        with pytest.raises(expected_exception=PyToRustValueMappingError):
            await conn.execute(
                querystring=(f"SELECT * FROM {table_name} " "WHERE name = $(name)p"),  # noqa: ISC001
                parameters={"mistake": "wow"},
            )
