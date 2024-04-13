import datetime
import uuid
from ipaddress import IPv4Address
from typing import Any

import pytest
from tests.conftest import DefaultPydanticModel, DefaultPythonModelClass

from psqlpy import ConnectionPool
from psqlpy.extra_types import (
    BigInt,
    Integer,
    PyJSON,
    PyJSONB,
    PyText,
    PyUUID,
    SmallInt,
)

pytestmark = pytest.mark.anyio
now_datetime = datetime.datetime.now()
now_datetime_with_tz = datetime.datetime(
    2024,
    4,
    13,
    17,
    3,
    46,
    142574,
    tzinfo=datetime.timezone.utc,
)
uuid_ = uuid.uuid4()


async def test_as_class(
    psql_pool: ConnectionPool,
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


@pytest.mark.parametrize(
    ["postgres_type", "py_value", "expected_deserialized"],
    (
        ("BYTEA", b"Bytes", [66, 121, 116, 101, 115]),
        ("VARCHAR", "Some String", "Some String"),
        ("TEXT", "Some String", "Some String"),
        ("BOOL", True, True),
        ("INT2", SmallInt(12), 12),
        ("INT4", Integer(121231231), 121231231),
        ("INT8", BigInt(99999999999999999), 99999999999999999),
        ("FLOAT4", 32.12329864501953, 32.12329864501953),
        ("DATE", now_datetime.date(), now_datetime.date()),
        ("TIME", now_datetime.time(), now_datetime.time()),
        ("TIMESTAMP", now_datetime, now_datetime),
        ("TIMESTAMPTZ", now_datetime_with_tz, now_datetime_with_tz),
        ("UUID", PyUUID(str(uuid_)), str(uuid_)),
        ("INET", IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")),
        (
            "JSONB",
            {
                "test": ["something", 123, "here"],
                "nested": ["JSON"],
            },
            {
                "test": ["something", 123, "here"],
                "nested": ["JSON"],
            },
        ),
        (
            "JSONB",
            PyJSONB([{"array": "json"}, {"one more": "test"}]),
            [{"array": "json"}, {"one more": "test"}],
        ),
        (
            "JSON",
            {
                "test": ["something", 123, "here"],
                "nested": ["JSON"],
            },
            {
                "test": ["something", 123, "here"],
                "nested": ["JSON"],
            },
        ),
        (
            "JSON",
            PyJSON([{"array": "json"}, {"one more": "test"}]),
            [{"array": "json"}, {"one more": "test"}],
        ),
        (
            "VARCHAR ARRAY",
            ["Some String", "Some String"],
            ["Some String", "Some String"],
        ),
        (
            "TEXT ARRAY",
            [PyText("Some String"), PyText("Some String")],
            ["Some String", "Some String"],
        ),
        ("BOOL ARRAY", [True, False], [True, False]),
        ("INT2 ARRAY", [SmallInt(12), SmallInt(100)], [12, 100]),
        ("INT4 ARRAY", [Integer(121231231), Integer(121231231)], [121231231, 121231231]),
        (
            "INT8 ARRAY",
            [BigInt(99999999999999999), BigInt(99999999999999999)],
            [99999999999999999, 99999999999999999],
        ),
        (
            "FLOAT4 ARRAY",
            [32.12329864501953, 32.12329864501953],
            [32.12329864501953, 32.12329864501953],
        ),
        (
            "DATE ARRAY",
            [now_datetime.date(), now_datetime.date()],
            [now_datetime.date(), now_datetime.date()],
        ),
        (
            "TIME ARRAY",
            [now_datetime.time(), now_datetime.time()],
            [now_datetime.time(), now_datetime.time()],
        ),
        ("TIMESTAMP ARRAY", [now_datetime, now_datetime], [now_datetime, now_datetime]),
        (
            "TIMESTAMPTZ ARRAY",
            [now_datetime_with_tz, now_datetime_with_tz],
            [now_datetime_with_tz, now_datetime_with_tz],
        ),
        (
            "UUID ARRAY",
            [PyUUID(str(uuid_)), PyUUID(str(uuid_))],
            [str(uuid_), str(uuid_)],
        ),
        (
            "INET ARRAY",
            [IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")],
            [IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")],
        ),
        (
            "JSONB ARRAY",
            [
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
            ],
            [
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
            ],
        ),
        (
            "JSONB ARRAY",
            [
                PyJSONB([{"array": "json"}, {"one more": "test"}]),
                PyJSONB([{"array": "json"}, {"one more": "test"}]),
            ],
            [
                [{"array": "json"}, {"one more": "test"}],
                [{"array": "json"}, {"one more": "test"}],
            ],
        ),
        (
            "JSON ARRAY",
            [
                PyJSON(
                    {
                        "test": ["something", 123, "here"],
                        "nested": ["JSON"],
                    },
                ),
                PyJSON(
                    {
                        "test": ["something", 123, "here"],
                        "nested": ["JSON"],
                    },
                ),
            ],
            [
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
            ],
        ),
        (
            "JSON ARRAY",
            [
                PyJSON([{"array": "json"}, {"one more": "test"}]),
                PyJSON([{"array": "json"}, {"one more": "test"}]),
            ],
            [
                [{"array": "json"}, {"one more": "test"}],
                [{"array": "json"}, {"one more": "test"}],
            ],
        ),
    ),
)
async def test_deserialization_rust_into_python(
    psql_pool: ConnectionPool,
    postgres_type: str,
    py_value: Any,
    expected_deserialized: Any,
) -> None:
    """Test how types can cast from Python and to Python."""
    await psql_pool.execute("DROP TABLE IF EXISTS for_test")
    create_table_query = f"""
    CREATE TABLE for_test (test_field {postgres_type})
    """
    insert_data_query = """
    INSERT INTO for_test VALUES ($1)
    """
    await psql_pool.execute(querystring=create_table_query)
    await psql_pool.execute(
        querystring=insert_data_query,
        parameters=[py_value],
    )

    raw_result = await psql_pool.execute(
        querystring="SELECT test_field FROM for_test",
    )

    assert raw_result.result()[0]["test_field"] == expected_deserialized
