import os
import random
from typing import AsyncGenerator

import pytest
from pydantic import BaseModel

from psqlpy import Cursor, PSQLPool


class DefaultPydanticModel(BaseModel):
    """Validation model for test data based on Pydantic."""

    id: int
    name: str


class DefaultPythonModelClass:
    """Validation model for test data based on default Python class."""

    def __init__(self, id: int, name: str) -> None:
        self.id = id
        self.name = name


@pytest.fixture()
def anyio_backend() -> str:
    """
    Anyio backend.

    Backend for anyio pytest plugin.
    :return: backend name.
    """
    return "asyncio"


def random_string(length: int = 10) -> str:
    return "".join(random.choice("AbCdEfG") for _ in range(length))


@pytest.fixture()
def postgres_host() -> str:
    return os.environ.get("POSTGRES_HOST", "localhost")


@pytest.fixture()
def postgres_user() -> str:
    return os.environ.get("POSTGRES_USER", "postgres")


@pytest.fixture()
def postgres_password() -> str:
    return os.environ.get("POSTGRES_PASSWORD", "postgres")


@pytest.fixture()
def postgres_port() -> int:
    return int(os.environ.get("POSTGRES_PORT", 5432))


@pytest.fixture()
def postgres_dbname() -> str:
    return os.environ.get("POSTGRES_DBNAME", "psqlpy_test")


@pytest.fixture()
def table_name() -> str:
    return random_string()


@pytest.fixture()
def number_database_records() -> int:
    return random.randint(10, 35)


@pytest.fixture()
async def psql_pool(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
) -> PSQLPool:
    pg_pool = PSQLPool(
        username=postgres_user,
        password=postgres_password,
        host=postgres_host,
        port=postgres_port,
        db_name=postgres_dbname,
    )
    await pg_pool.startup()
    return pg_pool


@pytest.fixture(autouse=True)
async def create_deafult_data_for_tests(
    psql_pool: PSQLPool,
    table_name: str,
    number_database_records: int,
) -> AsyncGenerator[None, None]:
    await psql_pool.execute(
        f"CREATE TABLE {table_name} (id SERIAL, name VARCHAR(255))",
    )

    for table_id in range(1, number_database_records + 1):
        new_name = random_string()
        await psql_pool.execute(
            querystring=f"INSERT INTO {table_name} VALUES ($1, $2)",
            parameters=[table_id, new_name],
        )
    yield
    await psql_pool.execute(
        f"DROP TABLE {table_name}",
    )


@pytest.fixture()
async def test_cursor(
    psql_pool: PSQLPool,
    table_name: str,
) -> AsyncGenerator[Cursor, None]:
    connection = await psql_pool.connection()
    transaction = connection.transaction()
    await transaction.begin()
    cursor = await transaction.cursor(
        querystring=f"SELECT * FROM {table_name}",
    )
    yield cursor
    await transaction.commit()
