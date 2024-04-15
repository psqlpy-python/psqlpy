import datetime
import uuid
from ipaddress import IPv4Address
from typing import Any, Dict, List, Union

import pytest
from pydantic import BaseModel
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
async def test_deserialization_simple_into_python(
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


async def test_deserialization_composite_into_python(
    psql_pool: ConnectionPool,
) -> None:
    """Test that it's possible to deserialize custom postgresql type."""
    await psql_pool.execute("DROP TABLE IF EXISTS for_test")
    await psql_pool.execute("DROP TYPE IF EXISTS all_types")
    create_type_query = """
    CREATE type all_types AS (
        bytea_ BYTEA,
        varchar_ VARCHAR,
        text_ TEXT,
        bool_ BOOL,
        int2_ INT2,
        int4_ INT4,
        int8_ INT8,
        flaot4_ FLOAT4,
        date_ DATE,
        time_ TIME,
        timestamp_ TIMESTAMP,
        timestampz_ TIMESTAMPTZ,
        uuid_ UUID,
        inet_ INET,
        jsonb_ JSONB,
        json_ JSON,

        varchar_arr VARCHAR ARRAY,
        text_arr TEXT ARRAY,
        bool_arr BOOL ARRAY,
        int2_arr INT2 ARRAY,
        int4_arr INT4 ARRAY,
        int8_arr INT8 ARRAY,
        flaot4_arr FLOAT4 ARRAY,
        date_arr DATE ARRAY,
        time_arr TIME ARRAY,
        timestamp_arr TIMESTAMP ARRAY,
        timestampz_arr TIMESTAMPTZ ARRAY,
        uuid_arr UUID ARRAY,
        inet_arr INET ARRAY,
        jsonb_arr JSONB ARRAY,
        json_arr JSON ARRAY
    )
    """
    create_table_query = """
    CREATE table for_test (custom_type all_types)
    """

    await psql_pool.execute(
        querystring=create_type_query,
    )
    await psql_pool.execute(
        querystring=create_table_query,
    )
    await psql_pool.execute(
        querystring="INSERT INTO for_test VALUES (ROW($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31))",  # noqa: E501
        parameters=[
            b"Bytes",
            "Some String",
            PyText("Some String"),
            True,
            SmallInt(123),
            Integer(199),
            BigInt(10001),
            32.12329864501953,
            now_datetime.date(),
            now_datetime.time(),
            now_datetime,
            now_datetime_with_tz,
            PyUUID(str(uuid_)),
            IPv4Address("192.0.0.1"),
            {
                "test": ["something", 123, "here"],
                "nested": ["JSON"],
            },
            PyJSON(
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
            ),
            ["Some String", "Some String"],
            [PyText("Some String"), PyText("Some String")],
            [True, False],
            [SmallInt(123), SmallInt(321)],
            [Integer(123), Integer(321)],
            [BigInt(10001), BigInt(10001)],
            [32.12329864501953, 32.12329864501953],
            [now_datetime.date(), now_datetime.date()],
            [now_datetime.time(), now_datetime.time()],
            [now_datetime, now_datetime],
            [now_datetime_with_tz, now_datetime_with_tz],
            [PyUUID(str(uuid_)), PyUUID(str(uuid_))],
            [IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")],
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
        ],
    )

    class ValidateModelForCustomType(BaseModel):
        bytea_: List[int]
        varchar_: str
        text_: str
        bool_: bool
        int2_: int
        int4_: int
        int8_: int
        flaot4_: float
        date_: datetime.date
        time_: datetime.time
        timestamp_: datetime.datetime
        timestampz_: datetime.datetime
        uuid_: uuid.UUID
        inet_: IPv4Address
        jsonb_: Dict[str, List[Union[str, int, List[str]]]]
        json_: Dict[str, List[Union[str, int, List[str]]]]

        varchar_arr: List[str]
        text_arr: List[str]
        bool_arr: List[bool]
        int2_arr: List[int]
        int4_arr: List[int]
        int8_arr: List[int]
        flaot4_arr: List[float]
        date_arr: List[datetime.date]
        time_arr: List[datetime.time]
        timestamp_arr: List[datetime.datetime]
        timestampz_arr: List[datetime.datetime]
        uuid_arr: List[uuid.UUID]
        inet_arr: List[IPv4Address]
        jsonb_arr: List[Dict[str, List[Union[str, int, List[str]]]]]
        json_arr: List[Dict[str, List[Union[str, int, List[str]]]]]

    class TopLevelModel(BaseModel):
        custom_type: ValidateModelForCustomType

    query_result = await psql_pool.execute(
        "SELECT custom_type FROM for_test",
    )

    model_result = query_result.as_class(
        as_class=TopLevelModel,
    )

    assert isinstance(model_result[0], TopLevelModel)
