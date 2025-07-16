import os
import random
from typing import AsyncGenerator

import pytest
from psqlpy import ConnectionPool, Cursor
from psqlpy._internal import SslMode
from pydantic import BaseModel


class DefaultPydanticModel(BaseModel):
    """Validation model for test data based on Pydantic."""

    id: int
    name: str


class DefaultPythonModelClass:
    """Validation model for test data based on default Python class."""

    def __init__(self, id: int, name: str) -> None:
        self.id = id
        self.name = name


@pytest.fixture
def anyio_backend() -> str:
    """
    Anyio backend.

    Backend for anyio pytest plugin.
    :return: backend name.
    """
    return "asyncio"


def random_string(length: int = 10) -> str:
    return "".join(random.choice("AbCdEfG") for _ in range(length))


@pytest.fixture
def postgres_host() -> str:
    return os.environ.get("POSTGRES_HOST", "localhost")


@pytest.fixture
def postgres_user() -> str:
    return os.environ.get("POSTGRES_USER", "postgres")


@pytest.fixture
def postgres_password() -> str:
    return os.environ.get("POSTGRES_PASSWORD", "postgres")


@pytest.fixture
def postgres_port() -> int:
    return int(os.environ.get("POSTGRES_PORT", 5432))


@pytest.fixture
def postgres_dbname() -> str:
    return os.environ.get("POSTGRES_DBNAME", "psqlpy_test")


@pytest.fixture
def table_name() -> str:
    return random_string()


@pytest.fixture
def listener_table_name() -> str:
    return random_string()


@pytest.fixture
def map_parameters_table_name() -> str:
    return random_string()


@pytest.fixture
def number_database_records() -> int:
    return random.randint(10, 35)


@pytest.fixture
def ssl_cert_file() -> str:
    return os.environ.get(
        "POSTGRES_CERT_FILE",
        "/home/runner/work/_temp/pgdata/server.crt",
    )


@pytest.fixture
async def psql_pool(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
) -> ConnectionPool:
    return ConnectionPool(
        username=postgres_user,
        password=postgres_password,
        host=postgres_host,
        port=postgres_port,
        db_name=postgres_dbname,
    )


@pytest.fixture
async def psql_pool_with_cert_file(
    postgres_host: str,
    postgres_user: str,
    postgres_password: str,
    postgres_port: int,
    postgres_dbname: str,
    ssl_cert_file: str,
) -> ConnectionPool:
    return ConnectionPool(
        username=postgres_user,
        password=postgres_password,
        host=postgres_host,
        port=postgres_port,
        db_name=postgres_dbname,
        ssl_mode=SslMode.VerifyFull,
        ca_file=ssl_cert_file,
    )


@pytest.fixture(autouse=True)
async def create_default_data_for_tests(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> AsyncGenerator[None, None]:
    async with psql_pool.acquire() as connection:
        await connection.execute(
            f"CREATE TABLE {table_name} (id SERIAL, name VARCHAR(255))",
        )

        for table_id in range(1, number_database_records + 1):
            new_name = random_string()
            await connection.execute(
                querystring=f"INSERT INTO {table_name} VALUES ($1, $2)",
                parameters=[table_id, new_name],
            )

    yield

    async with psql_pool.acquire() as connection:
        await connection.execute(
            f"DROP TABLE {table_name}",
        )


@pytest.fixture
async def create_table_for_listener_tests(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> AsyncGenerator[None, None]:
    async with psql_pool.acquire() as connection:
        await connection.execute(
            f"CREATE TABLE {listener_table_name}"
            f"(id SERIAL, payload VARCHAR(255),"
            f"channel VARCHAR(255), process_id INT)",
        )

    yield

    async with psql_pool.acquire() as connection:
        await connection.execute(
            f"DROP TABLE {listener_table_name}",
        )


@pytest.fixture
async def create_table_for_map_parameters_test(
    psql_pool: ConnectionPool,
    map_parameters_table_name: str,
) -> AsyncGenerator[None, None]:
    async with psql_pool.acquire() as connection:
        await connection.execute(
            f"CREATE TABLE {map_parameters_table_name}"
            "(id SERIAL, name VARCHAR(255),surname VARCHAR(255), age SMALLINT)",
        )

    yield

    async with psql_pool.acquire() as connection:
        await connection.execute(
            f"DROP TABLE {map_parameters_table_name}",
        )


@pytest.fixture
async def test_cursor(
    psql_pool: ConnectionPool,
    table_name: str,
) -> AsyncGenerator[Cursor, None]:
    async with psql_pool.acquire() as connection:
        transaction = connection.transaction()
        await transaction.begin()
        cursor = transaction.cursor(
            querystring=f"SELECT * FROM {table_name}",
        )
        await cursor.start()
        yield cursor
        await transaction.commit()
