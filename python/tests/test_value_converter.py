import datetime
import sys
import uuid
from decimal import Decimal
from enum import Enum
from ipaddress import IPv4Address
from typing import Any, Dict, List, Tuple, Union

import pytest
from psqlpy import ConnectionPool
from psqlpy.exceptions import PyToRustValueMappingError
from psqlpy.extra_types import (
    JSON,
    JSONB,
    BigInt,
    BoolArray,
    Box,
    BoxArray,
    Circle,
    CircleArray,
    CustomType,
    DateArray,
    DateTimeArray,
    DateTimeTZArray,
    Float32,
    Float64,
    Float64Array,
    Int16Array,
    Int32Array,
    Int64Array,
    Integer,
    IntervalArray,
    IpAddressArray,
    JSONArray,
    JSONBArray,
    Line,
    LineArray,
    LineSegment,
    LsegArray,
    MacAddr6,
    MacAddr8,
    Money,
    MoneyArray,
    NumericArray,
    Path,
    PathArray,
    Point,
    PointArray,
    SmallInt,
    Text,
    TextArray,
    TimeArray,
    UUIDArray,
    VarCharArray,
)
from pydantic import BaseModel
from typing_extensions import Annotated

from tests.conftest import DefaultPydanticModel, DefaultPythonModelClass

uuid_ = uuid.uuid4()
pytestmark = pytest.mark.anyio
now_datetime = datetime.datetime.now()  # noqa: DTZ005
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

now_datetime_with_tz_in_asia_jakarta = datetime.datetime(
    2024,
    4,
    13,
    17,
    3,
    46,
    142574,
    tzinfo=datetime.timezone.utc,
)
if sys.version_info >= (3, 9):
    import zoneinfo

    now_datetime_with_tz_in_asia_jakarta = datetime.datetime(
        2024,
        4,
        13,
        17,
        3,
        46,
        142574,
        tzinfo=zoneinfo.ZoneInfo(key="Asia/Jakarta"),
    )


async def test_as_class(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    """Test `as_class()` method."""
    connection = await psql_pool.connection()
    select_result = await connection.execute(
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
    ("postgres_type", "py_value", "expected_deserialized"),
    [
        ("BYTEA", b"Bytes", b"Bytes"),
        ("VARCHAR", "Some String", "Some String"),
        ("TEXT", "Some String", "Some String"),
        (
            "XML",
            """<?xml version="1.0"?><book><title>Manual</title><chapter>...</chapter></book>""",  # noqa: E501
            """<book><title>Manual</title><chapter>...</chapter></book>""",
        ),
        ("BOOL", True, True),
        ("INT2", SmallInt(12), 12),
        ("INT4", Integer(121231231), 121231231),
        ("INT8", BigInt(99999999999999999), 99999999999999999),
        ("MONEY", BigInt(99999999999999999), 99999999999999999),
        ("MONEY", Money(99999999999999999), 99999999999999999),
        ("NUMERIC(5, 2)", Decimal("120.12"), Decimal("120.12")),
        ("FLOAT8", 32.12329864501953, 32.12329864501953),
        ("FLOAT4", Float32(32.12329864501953), 32.12329864501953),
        ("FLOAT8", Float64(32.12329864501953), 32.12329864501953),
        ("DATE", now_datetime.date(), now_datetime.date()),
        ("TIME", now_datetime.time(), now_datetime.time()),
        ("TIMESTAMP", now_datetime, now_datetime),
        ("TIMESTAMPTZ", now_datetime_with_tz, now_datetime_with_tz),
        (
            "TIMESTAMPTZ",
            now_datetime_with_tz_in_asia_jakarta,
            now_datetime_with_tz_in_asia_jakarta,
        ),
        ("UUID", uuid_, str(uuid_)),
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
            JSONB([{"array": "json"}, {"one more": "test"}]),
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
            JSON([{"array": "json"}, {"one more": "test"}]),
            [{"array": "json"}, {"one more": "test"}],
        ),
        (
            "MACADDR",
            MacAddr6("08:00:2b:01:02:03"),
            "08:00:2B:01:02:03",
        ),
        (
            "MACADDR8",
            MacAddr8("08:00:2b:01:02:03:04:05"),
            "08:00:2B:01:02:03:04:05",
        ),
        ("POINT", Point([1.5, 2]), (1.5, 2.0)),
        ("POINT", Point({1.2, 2.3}), (1.2, 2.3)),
        ("POINT", Point((1.7, 2.8)), (1.7, 2.8)),
        ("BOX", Box([3.5, 3, 9, 9]), ((9.0, 9.0), (3.5, 3.0))),
        ("BOX", Box({(1, 2), (9, 9)}), ((9.0, 9.0), (1.0, 2.0))),
        ("BOX", Box(((1.7, 2.8), (9, 9))), ((9.0, 9.0), (1.7, 2.8))),
        (
            "PATH",
            Path([(3.5, 3), (9, 9), (8, 8)]),
            [(3.5, 3.0), (9.0, 9.0), (8.0, 8.0)],
        ),
        (
            "PATH",
            Path(((1.7, 2.8), (3.3, 2.5), (9, 9), (1.7, 2.8))),
            ((1.7, 2.8), (3.3, 2.5), (9.0, 9.0), (1.7, 2.8)),
        ),
        ("LINE", Line([-2, 1, 2]), (-2.0, 1.0, 2.0)),
        ("LINE", Line([1, -2, 3]), (1.0, -2.0, 3.0)),
        ("LSEG", LineSegment({(1, 2), (9, 9)}), [(1.0, 2.0), (9.0, 9.0)]),
        ("LSEG", LineSegment(((1.7, 2.8), (9, 9))), [(1.7, 2.8), (9.0, 9.0)]),
        (
            "CIRCLE",
            Circle((1.7, 2.8, 3)),
            ((1.7, 2.8), 3.0),
        ),
        (
            "CIRCLE",
            Circle([1, 2.8, 3]),
            ((1.0, 2.8), 3.0),
        ),
        (
            "INTERVAL",
            datetime.timedelta(days=100, microseconds=100),
            datetime.timedelta(days=100, microseconds=100),
        ),
        (
            "VARCHAR ARRAY",
            ["Some String", "Some String"],
            ["Some String", "Some String"],
        ),
        (
            "TEXT ARRAY",
            [Text("Some String"), Text("Some String")],
            ["Some String", "Some String"],
        ),
        ("BOOL ARRAY", [True, False], [True, False]),
        ("BOOL ARRAY", [[True], [False]], [[True], [False]]),
        ("INT2 ARRAY", [SmallInt(12), SmallInt(100)], [12, 100]),
        ("INT2 ARRAY", [[SmallInt(12)], [SmallInt(100)]], [[12], [100]]),
        ("INT4 ARRAY", [Integer(121231231), Integer(121231231)], [121231231, 121231231]),
        (
            "INT4 ARRAY",
            [[Integer(121231231)], [Integer(121231231)]],
            [[121231231], [121231231]],
        ),
        (
            "INT8 ARRAY",
            [BigInt(99999999999999999), BigInt(99999999999999999)],
            [99999999999999999, 99999999999999999],
        ),
        (
            "INT8 ARRAY",
            [[BigInt(99999999999999999)], [BigInt(99999999999999999)]],
            [[99999999999999999], [99999999999999999]],
        ),
        (
            "MONEY ARRAY",
            [Money(99999999999999999), Money(99999999999999999)],
            [99999999999999999, 99999999999999999],
        ),
        (
            "MONEY ARRAY",
            [[Money(99999999999999999)], [Money(99999999999999999)]],
            [[99999999999999999], [99999999999999999]],
        ),
        (
            "NUMERIC(5, 2) ARRAY",
            [Decimal("121.23"), Decimal("188.99")],
            [Decimal("121.23"), Decimal("188.99")],
        ),
        (
            "NUMERIC(5, 2) ARRAY",
            [[Decimal("121.23")], [Decimal("188.99")]],
            [[Decimal("121.23")], [Decimal("188.99")]],
        ),
        (
            "FLOAT8 ARRAY",
            [32.12329864501953, 32.12329864501953],
            [32.12329864501953, 32.12329864501953],
        ),
        (
            "FLOAT8 ARRAY",
            [[32.12329864501953], [32.12329864501953]],
            [[32.12329864501953], [32.12329864501953]],
        ),
        (
            "DATE ARRAY",
            [now_datetime.date(), now_datetime.date()],
            [now_datetime.date(), now_datetime.date()],
        ),
        (
            "DATE ARRAY",
            [[now_datetime.date()], [now_datetime.date()]],
            [[now_datetime.date()], [now_datetime.date()]],
        ),
        (
            "TIME ARRAY",
            [now_datetime.time(), now_datetime.time()],
            [now_datetime.time(), now_datetime.time()],
        ),
        (
            "TIME ARRAY",
            [[now_datetime.time()], [now_datetime.time()]],
            [[now_datetime.time()], [now_datetime.time()]],
        ),
        ("TIMESTAMP ARRAY", [now_datetime, now_datetime], [now_datetime, now_datetime]),
        (
            "TIMESTAMP ARRAY",
            [[now_datetime], [now_datetime]],
            [[now_datetime], [now_datetime]],
        ),
        (
            "TIMESTAMPTZ ARRAY",
            [now_datetime_with_tz, now_datetime_with_tz],
            [now_datetime_with_tz, now_datetime_with_tz],
        ),
        (
            "TIMESTAMPTZ ARRAY",
            [now_datetime_with_tz, now_datetime_with_tz_in_asia_jakarta],
            [now_datetime_with_tz, now_datetime_with_tz_in_asia_jakarta],
        ),
        (
            "TIMESTAMPTZ ARRAY",
            [[now_datetime_with_tz], [now_datetime_with_tz]],
            [[now_datetime_with_tz], [now_datetime_with_tz]],
        ),
        (
            "UUID ARRAY",
            [uuid_, uuid_],
            [str(uuid_), str(uuid_)],
        ),
        (
            "UUID ARRAY",
            [[uuid_], [uuid_]],
            [[str(uuid_)], [str(uuid_)]],
        ),
        (
            "INET ARRAY",
            [IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")],
            [IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")],
        ),
        (
            "INET ARRAY",
            [[IPv4Address("192.0.0.1")], [IPv4Address("192.0.0.1")]],
            [[IPv4Address("192.0.0.1")], [IPv4Address("192.0.0.1")]],
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
                [
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
                ],
            ],
            [
                [
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
                ],
            ],
        ),
        (
            "JSONB ARRAY",
            [
                JSONB([{"array": "json"}, {"one more": "test"}]),
                JSONB([{"array": "json"}, {"one more": "test"}]),
            ],
            [
                [{"array": "json"}, {"one more": "test"}],
                [{"array": "json"}, {"one more": "test"}],
            ],
        ),
        (
            "JSONB ARRAY",
            [
                JSONB([[{"array": "json"}], [{"one more": "test"}]]),
                JSONB([[{"array": "json"}], [{"one more": "test"}]]),
            ],
            [
                [[{"array": "json"}], [{"one more": "test"}]],
                [[{"array": "json"}], [{"one more": "test"}]],
            ],
        ),
        (
            "JSON ARRAY",
            [
                JSON(
                    {
                        "test": ["something", 123, "here"],
                        "nested": ["JSON"],
                    },
                ),
                JSON(
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
                [
                    JSON(
                        {
                            "test": ["something", 123, "here"],
                            "nested": ["JSON"],
                        },
                    ),
                ],
                [
                    JSON(
                        {
                            "test": ["something", 123, "here"],
                            "nested": ["JSON"],
                        },
                    ),
                ],
            ],
            [
                [
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
                ],
            ],
        ),
        (
            "JSON ARRAY",
            [
                JSON([{"array": "json"}, {"one more": "test"}]),
                JSON([{"array": "json"}, {"one more": "test"}]),
            ],
            [
                [{"array": "json"}, {"one more": "test"}],
                [{"array": "json"}, {"one more": "test"}],
            ],
        ),
        (
            "JSON ARRAY",
            [
                JSON([[{"array": "json"}], [{"one more": "test"}]]),
                JSON([[{"array": "json"}], [{"one more": "test"}]]),
            ],
            [
                [[{"array": "json"}], [{"one more": "test"}]],
                [[{"array": "json"}], [{"one more": "test"}]],
            ],
        ),
        (
            "POINT ARRAY",
            [
                Point([1.5, 2]),
                Point([2, 3]),
            ],
            [
                (1.5, 2.0),
                (2.0, 3.0),
            ],
        ),
        (
            "POINT ARRAY",
            [
                [Point([1.5, 2])],
                [Point([2, 3])],
            ],
            [
                [(1.5, 2.0)],
                [(2.0, 3.0)],
            ],
        ),
        (
            "BOX ARRAY",
            [
                Box([3.5, 3, 9, 9]),
                Box([8.5, 8, 9, 9]),
            ],
            [
                ((9.0, 9.0), (3.5, 3.0)),
                ((9.0, 9.0), (8.5, 8.0)),
            ],
        ),
        (
            "BOX ARRAY",
            [
                [Box([3.5, 3, 9, 9])],
                [Box([8.5, 8, 9, 9])],
            ],
            [
                [((9.0, 9.0), (3.5, 3.0))],
                [((9.0, 9.0), (8.5, 8.0))],
            ],
        ),
        (
            "PATH ARRAY",
            [
                Path([(3.5, 3), (9, 9), (8, 8)]),
                Path([(3.5, 3), (6, 6), (3.5, 3)]),
            ],
            [
                [(3.5, 3.0), (9.0, 9.0), (8.0, 8.0)],
                ((3.5, 3.0), (6.0, 6.0), (3.5, 3.0)),
            ],
        ),
        (
            "PATH ARRAY",
            [
                [Path([(3.5, 3), (9, 9), (8, 8)])],
                [Path([(3.5, 3), (6, 6), (3.5, 3)])],
            ],
            [
                [[(3.5, 3.0), (9.0, 9.0), (8.0, 8.0)]],
                [((3.5, 3.0), (6.0, 6.0), (3.5, 3.0))],
            ],
        ),
        (
            "LINE ARRAY",
            [
                Line([-2, 1, 2]),
                Line([1, -2, 3]),
            ],
            [
                (-2.0, 1.0, 2.0),
                (1.0, -2.0, 3.0),
            ],
        ),
        (
            "LINE ARRAY",
            [
                [Line([-2, 1, 2])],
                [Line([1, -2, 3])],
            ],
            [
                [(-2.0, 1.0, 2.0)],
                [(1.0, -2.0, 3.0)],
            ],
        ),
        (
            "LSEG ARRAY",
            [
                LineSegment({(1, 2), (9, 9)}),
                LineSegment([(5.6, 3.1), (4, 5)]),
            ],
            [
                [(1.0, 2.0), (9.0, 9.0)],
                [(5.6, 3.1), (4.0, 5.0)],
            ],
        ),
        (
            "LSEG ARRAY",
            [
                [LineSegment({(1, 2), (9, 9)})],
                [LineSegment([(5.6, 3.1), (4, 5)])],
            ],
            [
                [[(1.0, 2.0), (9.0, 9.0)]],
                [[(5.6, 3.1), (4.0, 5.0)]],
            ],
        ),
        (
            "CIRCLE ARRAY",
            [
                Circle([1.7, 2.8, 3]),
                Circle([5, 1.8, 10]),
            ],
            [
                ((1.7, 2.8), 3.0),
                ((5.0, 1.8), 10.0),
            ],
        ),
        (
            "CIRCLE ARRAY",
            [
                [Circle([1.7, 2.8, 3])],
                [Circle([5, 1.8, 10])],
            ],
            [
                [((1.7, 2.8), 3.0)],
                [((5.0, 1.8), 10.0)],
            ],
        ),
        (
            "INTERVAL ARRAY",
            [
                datetime.timedelta(days=100, microseconds=100),
                datetime.timedelta(days=100, microseconds=100),
            ],
            [
                datetime.timedelta(days=100, microseconds=100),
                datetime.timedelta(days=100, microseconds=100),
            ],
        ),
    ],
)
async def test_deserialization_simple_into_python(
    psql_pool: ConnectionPool,
    postgres_type: str,
    py_value: Any,
    expected_deserialized: Any,
) -> None:
    """Test how types can cast from Python and to Python."""
    connection = await psql_pool.connection()
    await connection.execute("DROP TABLE IF EXISTS for_test")
    create_table_query = f"""
    CREATE TABLE for_test (test_field {postgres_type})
    """
    insert_data_query = """
    INSERT INTO for_test VALUES ($1)
    """
    await connection.execute(querystring=create_table_query)
    await connection.execute(
        querystring=insert_data_query,
        parameters=[py_value],
    )

    raw_result = await connection.execute(
        querystring="SELECT test_field FROM for_test",
    )

    assert raw_result.result()[0]["test_field"] == expected_deserialized


async def test_deserialization_composite_into_python(
    psql_pool: ConnectionPool,
) -> None:
    """Test that it's possible to deserialize custom postgresql type."""
    connection = await psql_pool.connection()
    await connection.execute("DROP TABLE IF EXISTS for_test")
    await connection.execute("DROP TYPE IF EXISTS all_types")
    await connection.execute("DROP TYPE IF EXISTS inner_type")
    await connection.execute("DROP TYPE IF EXISTS enum_type")
    await connection.execute("CREATE TYPE enum_type AS ENUM ('sad', 'ok', 'happy')")
    await connection.execute(
        "CREATE TYPE inner_type AS (inner_value VARCHAR, some_enum enum_type)",
    )
    create_type_query = """
    CREATE type all_types AS (
        bytea_ BYTEA,
        varchar_ VARCHAR,
        text_ TEXT,
        bool_ BOOL,
        int2_ INT2,
        int4_ INT4,
        int8_ INT8,
        float8_def_ FLOAT8,
        float4_ FLOAT4,
        float8_ FLOAT8,
        date_ DATE,
        time_ TIME,
        timestamp_ TIMESTAMP,
        timestampz_ TIMESTAMPTZ,
        uuid_ UUID,
        inet_ INET,
        jsonb_ JSONB,
        json_ JSON,
        point_ POINT,
        box_ BOX,
        path_ PATH,
        line_ LINE,
        lseg_ LSEG,
        circle_ CIRCLE,

        varchar_arr VARCHAR ARRAY,
        varchar_arr_mdim VARCHAR ARRAY,
        text_arr TEXT ARRAY,
        bool_arr BOOL ARRAY,
        int2_arr INT2 ARRAY,
        int4_arr INT4 ARRAY,
        int8_arr INT8 ARRAY,
        float8_arr FLOAT8 ARRAY,
        date_arr DATE ARRAY,
        time_arr TIME ARRAY,
        timestamp_arr TIMESTAMP ARRAY,
        timestampz_arr TIMESTAMPTZ ARRAY,
        uuid_arr UUID ARRAY,
        inet_arr INET ARRAY,
        jsonb_arr JSONB ARRAY,
        json_arr JSON ARRAY,
        test_inner_value inner_type,
        test_enum_type enum_type,
        point_arr POINT ARRAY,
        box_arr BOX ARRAY,
        path_arr PATH ARRAY,
        line_arr LINE ARRAY,
        lseg_arr LSEG ARRAY,
        circle_arr CIRCLE ARRAY
    )
    """
    create_table_query = """
    CREATE table for_test (custom_type all_types)
    """

    await connection.execute(
        querystring=create_type_query,
    )
    await connection.execute(
        querystring=create_table_query,
    )

    class TestEnum(Enum):
        OK = "ok"
        SAD = "sad"
        HAPPY = "happy"

    row_values = ", ".join([f"${index}" for index in range(1, 41)])
    row_values += ", ROW($41, $42), "
    row_values += ", ".join([f"${index}" for index in range(43, 50)])

    await connection.execute(
        querystring=f"INSERT INTO for_test VALUES (ROW({row_values}))",
        parameters=[
            b"Bytes",
            "Some String",
            Text("Some String"),
            True,
            SmallInt(123),
            Integer(199),
            BigInt(10001),
            32.12329864501953,
            Float32(32.12329864501953),
            Float64(32.12329864501953),
            now_datetime.date(),
            now_datetime.time(),
            now_datetime,
            now_datetime_with_tz,
            uuid_,
            IPv4Address("192.0.0.1"),
            {
                "test": ["something", 123, "here"],
                "nested": ["JSON"],
            },
            JSON(
                {
                    "test": ["something", 123, "here"],
                    "nested": ["JSON"],
                },
            ),
            Point({1.2, 2.3}),
            Box(((1.7, 2.8), (9, 9))),
            Path(((1.7, 2.8), (3.3, 2.5), (9, 9), (1.7, 2.8))),
            Line({-2, 1, 2}),
            LineSegment(((1.7, 2.8), (9, 9))),
            Circle([1.7, 2.8, 3]),
            ["Some String", "Some String"],
            [["Some String"], ["Some String"]],
            [Text("Some String"), Text("Some String")],
            [True, False],
            [SmallInt(123), SmallInt(321)],
            [Integer(123), Integer(321)],
            [BigInt(10001), BigInt(10001)],
            [32.12329864501953, 32.12329864501953],
            [now_datetime.date(), now_datetime.date()],
            [now_datetime.time(), now_datetime.time()],
            [now_datetime, now_datetime],
            [now_datetime_with_tz, now_datetime_with_tz],
            [uuid_, uuid_],
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
                JSON(
                    {
                        "test": ["something", 123, "here"],
                        "nested": ["JSON"],
                    },
                ),
                JSON(
                    {
                        "test": ["something", 123, "here"],
                        "nested": ["JSON"],
                    },
                ),
            ],
            "inner type value",
            "happy",
            TestEnum.OK,
            [
                Point([1.5, 2]),
                Point([2, 3]),
            ],
            [
                Box([3.5, 3, 9, 9]),
                Box([8.5, 8, 9, 9]),
            ],
            [
                Path([(3.5, 3), (9, 9), (8, 8)]),
                Path([(3.5, 3), (6, 6), (3.5, 3)]),
            ],
            [
                Line([-2, 1, 2]),
                Line([5.6, 4, 5]),
            ],
            [
                LineSegment({(1, 2), (9, 9)}),
                LineSegment([(5.6, 3.1), (4, 5)]),
            ],
            [
                Circle([1.7, 2.8, 3]),
                Circle([5, 1.8, 10]),
            ],
        ],
    )

    class ValidateModelForInnerValueType(BaseModel):
        inner_value: str
        some_enum: TestEnum

    class ValidateModelForCustomType(BaseModel):
        bytea_: bytes
        varchar_: str
        text_: str
        bool_: bool
        int2_: int
        int4_: int
        int8_: int
        float8_def_: float
        float4_: float
        float8_: float
        date_: datetime.date
        time_: datetime.time
        timestamp_: datetime.datetime
        timestampz_: datetime.datetime
        uuid_: uuid.UUID
        inet_: IPv4Address
        jsonb_: Dict[str, List[Union[str, int, List[str]]]]
        json_: Dict[str, List[Union[str, int, List[str]]]]
        point_: Tuple[float, float]
        box_: Tuple[Tuple[float, float], Tuple[float, float]]
        path_: List[Tuple[float, float]]
        line_: Annotated[List[float], 3]
        lseg_: Annotated[List[Tuple[float, float]], 2]
        circle_: Tuple[Tuple[float, float], float]

        varchar_arr: List[str]
        varchar_arr_mdim: List[List[str]]
        text_arr: List[str]
        bool_arr: List[bool]
        int2_arr: List[int]
        int4_arr: List[int]
        int8_arr: List[int]
        float8_arr: List[float]
        date_arr: List[datetime.date]
        time_arr: List[datetime.time]
        timestamp_arr: List[datetime.datetime]
        timestampz_arr: List[datetime.datetime]
        uuid_arr: List[uuid.UUID]
        inet_arr: List[IPv4Address]
        jsonb_arr: List[Dict[str, List[Union[str, int, List[str]]]]]
        json_arr: List[Dict[str, List[Union[str, int, List[str]]]]]
        point_arr: List[Tuple[float, float]]
        box_arr: List[Tuple[Tuple[float, float], Tuple[float, float]]]
        path_arr: List[List[Tuple[float, float]]]
        line_arr: List[Annotated[List[float], 3]]
        lseg_arr: List[Annotated[List[Tuple[float, float]], 2]]
        circle_arr: List[Tuple[Tuple[float, float], float]]

        test_inner_value: ValidateModelForInnerValueType
        test_enum_type: TestEnum

    class TopLevelModel(BaseModel):
        custom_type: ValidateModelForCustomType

    query_result = await connection.execute(
        "SELECT custom_type FROM for_test",
    )

    model_result = query_result.as_class(
        as_class=TopLevelModel,
    )

    assert isinstance(model_result[0], TopLevelModel)


async def test_enum_type(psql_pool: ConnectionPool) -> None:
    """Test that we can decode ENUM type from PostgreSQL."""

    class TestEnum(Enum):
        OK = "ok"
        SAD = "sad"
        HAPPY = "happy"

    class TestStrEnum(str, Enum):
        OK = "ok"
        SAD = "sad"
        HAPPY = "happy"

    connection = await psql_pool.connection()
    await connection.execute("DROP TABLE IF EXISTS for_test")
    await connection.execute("DROP TYPE IF EXISTS mood")
    await connection.execute(
        "CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy')",
    )
    await connection.execute(
        "CREATE TABLE for_test (test_mood mood, test_mood2 mood)",
    )

    await connection.execute(
        querystring="INSERT INTO for_test VALUES ($1, $2)",
        parameters=[TestEnum.HAPPY, TestEnum.OK],
    )

    qs_result = await connection.execute(
        "SELECT * FROM for_test",
    )
    assert qs_result.result()[0]["test_mood"] == TestEnum.HAPPY.value
    assert qs_result.result()[0]["test_mood"] != TestEnum.HAPPY
    assert qs_result.result()[0]["test_mood2"] == TestStrEnum.OK


async def test_custom_type_as_parameter(
    psql_pool: ConnectionPool,
) -> None:
    """Tests that we can use `PyCustomType`."""
    connection = await psql_pool.connection()
    await connection.execute("DROP TABLE IF EXISTS for_test")
    await connection.execute(
        "CREATE TABLE for_test (nickname VARCHAR)",
    )

    await connection.execute(
        querystring="INSERT INTO for_test VALUES ($1)",
        parameters=[CustomType(b"Some Real Nickname")],
    )

    qs_result = await connection.execute(
        "SELECT * FROM for_test",
    )

    result = qs_result.result()
    assert result[0]["nickname"] == "Some Real Nickname"


async def test_custom_decoder(
    psql_pool: ConnectionPool,
) -> None:
    def point_encoder(point_bytes: bytes) -> str:  # noqa: ARG001
        return "Just An Example"

    async with psql_pool.acquire() as conn:
        await conn.execute("DROP TABLE IF EXISTS for_test")
        await conn.execute(
            "CREATE TABLE for_test (geo_point POINT)",
        )

        await conn.execute(
            "INSERT INTO for_test VALUES ('(1, 1)')",
        )

        qs_result = await conn.execute(
            "SELECT * FROM for_test",
        )
        result = qs_result.result(
            custom_decoders={
                "geo_point": point_encoder,
            },
        )

        assert result[0]["geo_point"] == "Just An Example"


async def test_row_factory_query_result(
    psql_pool: ConnectionPool,
    table_name: str,
    number_database_records: int,
) -> None:
    async with psql_pool.acquire() as conn:
        select_result = await conn.execute(
            f"SELECT * FROM {table_name}",
        )

        def row_factory(db_result: Dict[str, Any]) -> List[str]:
            return list(db_result.keys())

        as_row_factory = select_result.row_factory(
            row_factory=row_factory,
        )
        assert len(as_row_factory) == number_database_records

        assert isinstance(as_row_factory[0], list)


async def test_row_factory_single_query_result(
    psql_pool: ConnectionPool,
    table_name: str,
) -> None:
    async with psql_pool.acquire() as conn:
        select_result = await conn.fetch_row(
            f"SELECT * FROM {table_name} LIMIT 1",
        )

        def row_factory(db_result: Dict[str, Any]) -> List[str]:
            return list(db_result.keys())

        as_row_factory = select_result.row_factory(
            row_factory=row_factory,
        )
        expected_number_of_elements_in_result = 2
        assert len(as_row_factory) == expected_number_of_elements_in_result

        assert isinstance(as_row_factory, list)


async def test_incorrect_dimensions_array(
    psql_pool: ConnectionPool,
) -> None:
    async with psql_pool.acquire() as conn:
        await conn.execute("DROP TABLE IF EXISTS test_marr")
        await conn.execute("CREATE TABLE test_marr (var_array VARCHAR ARRAY)")

        with pytest.raises(expected_exception=PyToRustValueMappingError):
            await conn.execute(
                querystring="INSERT INTO test_marr VALUES ($1)",
                parameters=[
                    [
                        ["Len", "is", "Three"],
                        ["Len", "is", "Four", "Wow"],
                    ],
                ],
            )


async def test_empty_array(
    psql_pool: ConnectionPool,
) -> None:
    async with psql_pool.acquire() as conn:
        await conn.execute("DROP TABLE IF EXISTS test_earr")
        await conn.execute(
            """
            CREATE TABLE test_earr (
                id serial NOT NULL PRIMARY KEY,
                e_array text[] NOT NULL DEFAULT array[]::text[]
            )
            """,
        )

        await conn.execute("INSERT INTO test_earr(id) VALUES(2);")

        res = await conn.execute(
            "SELECT * FROM test_earr WHERE id = 2",
        )

        json_result = res.result()
        assert json_result
        assert not json_result[0]["e_array"]


@pytest.mark.parametrize(
    ("postgres_type", "py_value", "expected_deserialized"),
    [
        (
            "VARCHAR ARRAY",
            VarCharArray(["Some String", "Some String"]),
            ["Some String", "Some String"],
        ),
        (
            "VARCHAR ARRAY",
            VarCharArray([]),
            [],
        ),
        (
            "TEXT ARRAY",
            TextArray([]),
            [],
        ),
        (
            "TEXT ARRAY",
            TextArray([Text("Some String"), Text("Some String")]),
            ["Some String", "Some String"],
        ),
        ("BOOL ARRAY", BoolArray([]), []),
        ("BOOL ARRAY", BoolArray([True, False]), [True, False]),
        ("BOOL ARRAY", BoolArray([[True], [False]]), [[True], [False]]),
        ("INT2 ARRAY", Int16Array([]), []),
        ("INT2 ARRAY", Int16Array([SmallInt(12), SmallInt(100)]), [12, 100]),
        ("INT2 ARRAY", Int16Array([[SmallInt(12)], [SmallInt(100)]]), [[12], [100]]),
        (
            "INT4 ARRAY",
            Int32Array([Integer(121231231), Integer(121231231)]),
            [121231231, 121231231],
        ),
        (
            "INT4 ARRAY",
            Int32Array([[Integer(121231231)], [Integer(121231231)]]),
            [[121231231], [121231231]],
        ),
        (
            "INT8 ARRAY",
            Int64Array([BigInt(99999999999999999), BigInt(99999999999999999)]),
            [99999999999999999, 99999999999999999],
        ),
        (
            "INT8 ARRAY",
            Int64Array([[BigInt(99999999999999999)], [BigInt(99999999999999999)]]),
            [[99999999999999999], [99999999999999999]],
        ),
        (
            "MONEY ARRAY",
            MoneyArray([Money(99999999999999999), Money(99999999999999999)]),
            [99999999999999999, 99999999999999999],
        ),
        (
            "MONEY ARRAY",
            MoneyArray([[Money(99999999999999999)], [Money(99999999999999999)]]),
            [[99999999999999999], [99999999999999999]],
        ),
        (
            "NUMERIC(5, 2) ARRAY",
            NumericArray([Decimal("121.23"), Decimal("188.99")]),
            [Decimal("121.23"), Decimal("188.99")],
        ),
        (
            "NUMERIC(5, 2) ARRAY",
            NumericArray([[Decimal("121.23")], [Decimal("188.99")]]),
            [[Decimal("121.23")], [Decimal("188.99")]],
        ),
        (
            "FLOAT8 ARRAY",
            Float64Array([32.12329864501953, 32.12329864501953]),
            [32.12329864501953, 32.12329864501953],
        ),
        (
            "FLOAT8 ARRAY",
            Float64Array([[32.12329864501953], [32.12329864501953]]),
            [[32.12329864501953], [32.12329864501953]],
        ),
        (
            "DATE ARRAY",
            DateArray([now_datetime.date(), now_datetime.date()]),
            [now_datetime.date(), now_datetime.date()],
        ),
        (
            "DATE ARRAY",
            DateArray([[now_datetime.date()], [now_datetime.date()]]),
            [[now_datetime.date()], [now_datetime.date()]],
        ),
        (
            "TIME ARRAY",
            TimeArray([now_datetime.time(), now_datetime.time()]),
            [now_datetime.time(), now_datetime.time()],
        ),
        (
            "TIME ARRAY",
            TimeArray([[now_datetime.time()], [now_datetime.time()]]),
            [[now_datetime.time()], [now_datetime.time()]],
        ),
        (
            "TIMESTAMP ARRAY",
            DateTimeArray([now_datetime, now_datetime]),
            [now_datetime, now_datetime],
        ),
        (
            "TIMESTAMP ARRAY",
            DateTimeArray([[now_datetime], [now_datetime]]),
            [[now_datetime], [now_datetime]],
        ),
        (
            "TIMESTAMPTZ ARRAY",
            DateTimeTZArray([now_datetime_with_tz, now_datetime_with_tz]),
            [now_datetime_with_tz, now_datetime_with_tz],
        ),
        (
            "TIMESTAMPTZ ARRAY",
            DateTimeTZArray([[now_datetime_with_tz], [now_datetime_with_tz]]),
            [[now_datetime_with_tz], [now_datetime_with_tz]],
        ),
        (
            "UUID ARRAY",
            UUIDArray([uuid_, uuid_]),
            [str(uuid_), str(uuid_)],
        ),
        (
            "UUID ARRAY",
            UUIDArray([[uuid_], [uuid_]]),
            [[str(uuid_)], [str(uuid_)]],
        ),
        (
            "INET ARRAY",
            IpAddressArray([IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")]),
            [IPv4Address("192.0.0.1"), IPv4Address("192.0.0.1")],
        ),
        (
            "INET ARRAY",
            IpAddressArray([[IPv4Address("192.0.0.1")], [IPv4Address("192.0.0.1")]]),
            [[IPv4Address("192.0.0.1")], [IPv4Address("192.0.0.1")]],
        ),
        (
            "JSONB ARRAY",
            JSONBArray(
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
            JSONBArray(
                [
                    [
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
                    ],
                ],
            ),
            [
                [
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
                ],
            ],
        ),
        (
            "JSONB ARRAY",
            JSONBArray(
                [
                    JSONB([{"array": "json"}, {"one more": "test"}]),
                    JSONB([{"array": "json"}, {"one more": "test"}]),
                ],
            ),
            [
                [{"array": "json"}, {"one more": "test"}],
                [{"array": "json"}, {"one more": "test"}],
            ],
        ),
        (
            "JSONB ARRAY",
            JSONBArray(
                [
                    JSONB([[{"array": "json"}], [{"one more": "test"}]]),
                    JSONB([[{"array": "json"}], [{"one more": "test"}]]),
                ],
            ),
            [
                [[{"array": "json"}], [{"one more": "test"}]],
                [[{"array": "json"}], [{"one more": "test"}]],
            ],
        ),
        (
            "JSON ARRAY",
            JSONArray(
                [
                    JSON(
                        {
                            "test": ["something", 123, "here"],
                            "nested": ["JSON"],
                        },
                    ),
                    JSON(
                        {
                            "test": ["something", 123, "here"],
                            "nested": ["JSON"],
                        },
                    ),
                ],
            ),
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
            JSONArray(
                [
                    [
                        JSON(
                            {
                                "test": ["something", 123, "here"],
                                "nested": ["JSON"],
                            },
                        ),
                    ],
                    [
                        JSON(
                            {
                                "test": ["something", 123, "here"],
                                "nested": ["JSON"],
                            },
                        ),
                    ],
                ],
            ),
            [
                [
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
                ],
            ],
        ),
        (
            "JSON ARRAY",
            JSONArray(
                [
                    JSON([{"array": "json"}, {"one more": "test"}]),
                    JSON([{"array": "json"}, {"one more": "test"}]),
                ],
            ),
            [
                [{"array": "json"}, {"one more": "test"}],
                [{"array": "json"}, {"one more": "test"}],
            ],
        ),
        (
            "JSON ARRAY",
            JSONArray(
                [
                    JSON([[{"array": "json"}], [{"one more": "test"}]]),
                    JSON([[{"array": "json"}], [{"one more": "test"}]]),
                ],
            ),
            [
                [[{"array": "json"}], [{"one more": "test"}]],
                [[{"array": "json"}], [{"one more": "test"}]],
            ],
        ),
        (
            "POINT ARRAY",
            PointArray(
                [
                    Point([1.5, 2]),
                    Point([2, 3]),
                ],
            ),
            [
                (1.5, 2.0),
                (2.0, 3.0),
            ],
        ),
        (
            "POINT ARRAY",
            PointArray(
                [
                    [Point([1.5, 2])],
                    [Point([2, 3])],
                ],
            ),
            [
                [(1.5, 2.0)],
                [(2.0, 3.0)],
            ],
        ),
        (
            "BOX ARRAY",
            BoxArray(
                [
                    Box([3.5, 3, 9, 9]),
                    Box([8.5, 8, 9, 9]),
                ],
            ),
            [
                ((9.0, 9.0), (3.5, 3.0)),
                ((9.0, 9.0), (8.5, 8.0)),
            ],
        ),
        (
            "BOX ARRAY",
            BoxArray(
                [
                    [Box([3.5, 3, 9, 9])],
                    [Box([8.5, 8, 9, 9])],
                ],
            ),
            [
                [((9.0, 9.0), (3.5, 3.0))],
                [((9.0, 9.0), (8.5, 8.0))],
            ],
        ),
        (
            "PATH ARRAY",
            PathArray(
                [
                    Path([(3.5, 3), (9, 9), (8, 8)]),
                    Path([(3.5, 3), (6, 6), (3.5, 3)]),
                ],
            ),
            [
                [(3.5, 3.0), (9.0, 9.0), (8.0, 8.0)],
                ((3.5, 3.0), (6.0, 6.0), (3.5, 3.0)),
            ],
        ),
        (
            "PATH ARRAY",
            PathArray(
                [
                    [Path([(3.5, 3), (9, 9), (8, 8)])],
                    [Path([(3.5, 3), (6, 6), (3.5, 3)])],
                ],
            ),
            [
                [[(3.5, 3.0), (9.0, 9.0), (8.0, 8.0)]],
                [((3.5, 3.0), (6.0, 6.0), (3.5, 3.0))],
            ],
        ),
        (
            "LINE ARRAY",
            LineArray(
                [
                    Line([-2, 1, 2]),
                    Line([1, -2, 3]),
                ],
            ),
            [
                (-2.0, 1.0, 2.0),
                (1.0, -2.0, 3.0),
            ],
        ),
        (
            "LINE ARRAY",
            LineArray(
                [
                    [Line([-2, 1, 2])],
                    [Line([1, -2, 3])],
                ],
            ),
            [
                [(-2.0, 1.0, 2.0)],
                [(1.0, -2.0, 3.0)],
            ],
        ),
        (
            "LSEG ARRAY",
            LsegArray(
                [
                    LineSegment({(1, 2), (9, 9)}),
                    LineSegment([(5.6, 3.1), (4, 5)]),
                ],
            ),
            [
                [(1.0, 2.0), (9.0, 9.0)],
                [(5.6, 3.1), (4.0, 5.0)],
            ],
        ),
        (
            "LSEG ARRAY",
            LsegArray(
                [
                    [LineSegment({(1, 2), (9, 9)})],
                    [LineSegment([(5.6, 3.1), (4, 5)])],
                ],
            ),
            [
                [[(1.0, 2.0), (9.0, 9.0)]],
                [[(5.6, 3.1), (4.0, 5.0)]],
            ],
        ),
        (
            "CIRCLE ARRAY",
            CircleArray(
                [
                    Circle([1.7, 2.8, 3]),
                    Circle([5, 1.8, 10]),
                ],
            ),
            [
                ((1.7, 2.8), 3.0),
                ((5.0, 1.8), 10.0),
            ],
        ),
        (
            "CIRCLE ARRAY",
            CircleArray(
                [
                    [Circle([1.7, 2.8, 3])],
                    [Circle([5, 1.8, 10])],
                ],
            ),
            [
                [((1.7, 2.8), 3.0)],
                [((5.0, 1.8), 10.0)],
            ],
        ),
        (
            "INTERVAL ARRAY",
            IntervalArray(
                [
                    [datetime.timedelta(days=100, microseconds=100)],
                    [datetime.timedelta(days=100, microseconds=100)],
                ],
            ),
            [
                [datetime.timedelta(days=100, microseconds=100)],
                [datetime.timedelta(days=100, microseconds=100)],
            ],
        ),
    ],
)
async def test_array_types(
    psql_pool: ConnectionPool,
    postgres_type: str,
    py_value: Any,
    expected_deserialized: Any,
) -> None:
    async with psql_pool.acquire() as conn:
        await conn.execute("DROP TABLE IF EXISTS for_test")
        create_table_query = f"""
        CREATE TABLE for_test (test_field {postgres_type})
        """
        insert_data_query = """
        INSERT INTO for_test VALUES ($1)
        """
        await conn.execute(querystring=create_table_query)
        await conn.execute(
            querystring=insert_data_query,
            parameters=[py_value],
        )

        raw_result = await conn.execute(
            querystring="SELECT test_field FROM for_test",
        )

        assert raw_result.result()[0]["test_field"] == expected_deserialized
