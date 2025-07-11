import types
import typing
from enum import Enum
from io import BytesIO
from ipaddress import IPv4Address, IPv6Address
from typing import Any, Awaitable, Callable, Mapping, Sequence, TypeVar

from typing_extensions import Buffer, Self, TypeAlias

_CustomClass = TypeVar(
    "_CustomClass",
)
_RowFactoryRV = TypeVar(
    "_RowFactoryRV",
)

ParamsT: TypeAlias = Sequence[Any] | Mapping[str, Any] | None

class QueryResult:
    """Result."""

    @typing.overload
    def result(
        self: Self,
        as_tuple: typing.Literal[None] = None,
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> list[dict[str, Any]]: ...
    @typing.overload
    def result(
        self: Self,
        as_tuple: typing.Literal[False],
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> list[dict[str, Any]]: ...
    @typing.overload
    def result(
        self: Self,
        as_tuple: typing.Literal[True],
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> list[tuple[typing.Any, ...]]: ...
    @typing.overload
    def result(
        self: Self,
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
        as_tuple: bool | None = None,
    ) -> list[dict[str, Any]]:
        """Return result from database.

        By default it returns result as a list of dicts.

        `custom_decoders` must be used when you use
        PostgreSQL Type which isn't supported, read more in our docs.
        """

    def as_class(
        self: Self,
        as_class: Callable[..., _CustomClass],
    ) -> list[_CustomClass]:
        """Convert results to passed class.

        The main goal of this method is pydantic,
        msgspec and dataclasses support.

        ### Parameters:
        - `as_class`: Any callable python class for the results.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        class ExampleOfAsClass:
            def __init__(self, username: str) -> None:
                self.username = username

        async def main() -> None:
            db_pool = PSQLPool()
            query_result: QueryResult = await db_pool.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            class_results: List[ExampleOfAsClass] = query_result.as_class(
                as_class=ExampleOfAsClass,
            )
        ```
        """

    def row_factory(
        self,
        row_factory: Callable[[dict[str, Any]], _RowFactoryRV],
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> list[_RowFactoryRV]:
        """Use custom function to convert results from database.

        `custom_decoders` must be used when you use
        PostgreSQL Type isn't supported, read more in the docs.

        Argument order: firstly we apply `custom_decoders` (if specified),
        then we apply `row_factory`.

        ### Parameters:
        - `row_factory`: function which takes `dict[str, Any]` as an argument.
        - `custom_decoders`: functions for custom decoding.

        ### Returns:
        List of type that return passed `row_factory`.
        """

class SingleQueryResult:
    """Single result."""

    @typing.overload
    def result(
        self: Self,
        as_tuple: typing.Literal[None] = None,
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> dict[str, Any]: ...
    @typing.overload
    def result(
        self: Self,
        as_tuple: typing.Literal[False],
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> dict[str, Any]: ...
    @typing.overload
    def result(
        self: Self,
        as_tuple: typing.Literal[True],
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> tuple[typing.Any, ...]: ...
    @typing.overload
    def result(
        self: Self,
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
        as_tuple: bool | None = None,
    ) -> dict[Any, Any]:
        """Return result from database.

        By default it returns result as a dict.

        `custom_decoders` must be used when you use
        PostgreSQL Type which isn't supported, read more in our docs.
        """

    def as_class(
        self: Self,
        as_class: Callable[..., _CustomClass],
    ) -> list[_CustomClass]:
        """Convert results to passed class.

        The main goal of this method is pydantic,
        msgspec and dataclasses support.

        ### Parameters:
        - `as_class`: Any callable python class for the results.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        class ExampleOfAsClass:
            def __init__(self, username: str) -> None:
                self.username = username

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            async with connection.transaction() as trans:
                query_result: SingleQueryResult = await trans.fetch_row(
                    "SELECT username FROM users WHERE id = $1",
                    [100],
                )

            class_result: ExampleOfAsClass = query_result.as_class(
                as_class=ExampleOfAsClass,
            )
        ```
        """

    def row_factory(
        self,
        row_factory: Callable[[dict[str, Any]], _RowFactoryRV],
        custom_decoders: dict[str, Callable[[bytes], Any]] | None = None,
    ) -> _RowFactoryRV:
        """Use custom function to convert results from database.

        `custom_decoders` must be used when you use
        PostgreSQL Type isn't supported, read more in our docs.

        Argument order: firstly we apply `custom_decoders` (if specified),
        then we apply `row_factory`.

        ### Parameters:
        - `row_factory`: function which takes `list[dict[str, Any]]` as an argument.
        - `custom_decoders`: functions for custom decoding.

        ### Returns:
        Type that return passed function.
        """

class IsolationLevel(Enum):
    """Isolation Level for transactions."""

    ReadUncommitted = 1
    ReadCommitted = 2
    RepeatableRead = 3
    Serializable = 4

class LoadBalanceHosts(Enum):
    """Load balancing configuration."""

    # Make connection attempts to hosts in the order provided.
    Disable = 1
    # Make connection attempts to hosts in a random order.
    Random = 2

class TargetSessionAttrs(Enum):
    """Properties required of a session."""

    # No special properties are required.
    Any = 1
    # The session must allow writes.
    ReadWrite = 2
    # The session allow only reads.
    ReadOnly = 3

class ReadVariant(Enum):
    """Class for Read Variant for transaction."""

    ReadOnly = 1
    ReadWrite = 2

class ConnRecyclingMethod(Enum):
    """Possible methods of how a connection is recycled.

    The default is [`Fast`] which does not check the connection health or
    perform any clean-up queries.

    # Description:
    ## Fast:
    Only run [`is_closed()`] when recycling existing connections.

    Unless you have special needs this is a safe choice.

    ## Verified:
    Run [`is_closed()`] and execute a test query.

    This is slower, but guarantees that the database connection is ready to
    be used. Normally, [`is_closed()`] should be enough to filter
    out bad connections, but under some circumstances (i.e. hard-closed
    network connections) it's possible that [`is_closed()`]
    returns `false` while the connection is dead. You will receive an error
    on your first query then.

    ## Clean:
    Like [`Verified`] query method, but instead use the following sequence
    of statements which guarantees a pristine connection:
    ```sql
    CLOSE ALL;
    SET SESSION AUTHORIZATION DEFAULT;
    RESET ALL;
    UNLISTEN *;
    SELECT pg_advisory_unlock_all();
    DISCARD TEMP;
    DISCARD SEQUENCES;
    ```
    This is similar to calling `DISCARD ALL`. but doesn't call
    `DEALLOCATE ALL` and `DISCARD PLAN`, so that the statement cache is not
    rendered ineffective.
    """

    Fast = 1
    Verified = 2
    Clean = 3

class SslMode(Enum):
    """TLS configuration."""

    # Do not use TLS.
    Disable = 1
    # Pay the overhead of encryption if the server insists on it.
    Allow = 2
    # Attempt to connect with TLS but allow sessions without.
    Prefer = 3
    # Require the use of TLS.
    Require = 4
    # I want my data encrypted,
    # and I accept the overhead.
    # I want to be sure that I connect to a server that I trust.
    VerifyCa = 5
    # I want my data encrypted,
    # and I accept the overhead.
    # I want to be sure that I connect to a server I trust,
    # and that it's the one I specify.
    VerifyFull = 6

class KeepaliveConfig:
    """Config for configuring keepalive."""

    def __init__(self: Self, idle: int, interval: int, retries: int) -> None:
        """Initialize new config."""

class Cursor:
    """Represent binary cursor in a transaction.

    It can be used as an asynchronous iterator.
    """

    array_size: int
    cursor_name: str
    querystring: str
    parameters: ParamsT = None
    prepared: bool | None
    conn_dbname: str | None
    user: str | None
    host_addrs: list[str]
    hosts: list[str]
    ports: list[int]

    def __aiter__(self: Self) -> Self: ...
    async def __anext__(self: Self) -> QueryResult: ...
    async def __aenter__(self: Self) -> Self: ...
    async def __aexit__(
        self: Self,
        exception_type: type[BaseException] | None,
        exception: BaseException | None,
        traceback: types.TracebackType | None,
    ) -> None: ...
    async def start(self: Self) -> None:
        """Start the cursor.

        Execute DECLARE command for the cursor.
        """

    def close(self: Self) -> None:
        """Close the cursor.

        Execute CLOSE command for the cursor.
        """

    async def execute(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
    ) -> QueryResult:
        """Start cursor with querystring and parameters.

        Method should be used instead of context manager
        and `start` method.
        """

    async def fetchone(self: Self) -> QueryResult:
        """Return next one row from the cursor."""

    async def fetchmany(self: Self, size: int | None = None) -> QueryResult:
        """Return <size> rows from the cursor."""

    async def fetchall(self: Self, size: int | None = None) -> QueryResult:
        """Return all remaining rows from the cursor."""

class Transaction:
    """Single connection for executing queries.

    It represents transaction in database.

    You can create it only from `PSQLPool` with method
    `.transaction()`.
    """

    conn_dbname: str | None
    user: str | None
    host_addrs: list[str]
    hosts: list[str]
    ports: list[int]

    async def __aenter__(self: Self) -> Self: ...
    async def __aexit__(
        self: Self,
        exception_type: type[BaseException] | None,
        exception: BaseException | None,
        traceback: types.TracebackType | None,
    ) -> None: ...
    async def begin(self: Self) -> None:
        """Start the transaction.

        Execute `BEGIN`.

        `begin()` can be called only once per transaction.
        """

    async def commit(self: Self) -> None:
        """Commit the transaction.

        Execute `COMMIT`.

        `commit()` can be called only once per transaction.
        """

    async def rollback(self: Self) -> None:
        """Rollback all queries in the transaction.

        It can be done only one, after execution transaction marked
        as `done`.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()
            await transaction.execute(...)
            await transaction.rollback()
        ```
        """

    async def execute(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> QueryResult:
        """Execute the query.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()
            await transaction.begin()
            query_result: QueryResult = await transaction.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
            # You must call commit manually
            await transaction.commit()
        ```
        """

    async def execute_batch(
        self: Self,
        querystring: str,
    ) -> None:
        """
        Execute a sequence of SQL statements using the simple query protocol.

        Statements should be separated by semicolons.
        If an error occurs, execution of the sequence will stop at that point.
        This is intended for use when, for example,
        initializing a database schema.

        ### Parameters:
        - `querystring`: querystrings separated by semicolons.
        """

    async def execute_many(
        self: Self,
        querystring: str,
        parameters: Sequence[Sequence[Any]] | None = None,
        prepared: bool = True,
    ) -> None: ...
    """Execute query multiple times with different parameters.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()
            await transaction.begin()
            query_result: QueryResult = await transaction.execute_many(
                "INSERT INTO users (name, age) VALUES ($1, $2)",
                [["boba", 10], ["boba", 20]],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
            # You must call commit manually
            await transaction.commit()
        ```
        """
    async def fetch(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> QueryResult:
        """Fetch the result from database.

        It's the same as `execute` method, we made it because people are used
        to `fetch` method name.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.
        """

    async def fetch_row(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> SingleQueryResult:
        """Fetch exaclty single row from query.

        Query must return exactly one row, otherwise error will be raised.
        Querystring can contain `$<number>` parameters
        for converting them in the driver side.


        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()
            await transaction.begin()
            query_result: SingleQueryResult = await transaction.fetch_row(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: Dict[Any, Any] = query_result.result()
            # You must call commit manually
            await transaction.commit()
        ```
        """

    async def fetch_val(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> Any | None:
        """Execute the query and return first value of the first row.

        Returns an error if the query does not return exactly one row.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Raises
        - `RustPSQLDriverPyBaseError`: if the query does not
        return exactly one row

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()
            await transaction.begin()
            value: Any = await transaction.fetch_val(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
        ```
        """

    async def pipeline(
        self,
        queries: list[tuple[str, list[Any] | None]],
        prepared: bool = True,
    ) -> list[QueryResult]:
        """Execute queries in pipeline.

        Pipelining can improve performance in use cases in which multiple,
        independent queries need to be executed.
        In a traditional workflow,
        each query is sent to the server after the previous query completes.
        In contrast, pipelining allows the client to send all of the
        queries to the server up front, minimizing time spent
        by one side waiting for the other to finish sending data:
        ```
                            Sequential                              Pipelined
        | Client         | Server          |    | Client         | Server          |
        |----------------|-----------------|    |----------------|-----------------|
        | send query 1   |                 |    | send query 1   |                 |
        |                | process query 1 |    | send query 2   | process query 1 |
        | receive rows 1 |                 |    | send query 3   | process query 2 |
        | send query 2   |                 |    | receive rows 1 | process query 3 |
        |                | process query 2 |    | receive rows 2 |                 |
        | receive rows 2 |                 |    | receive rows 3 |                 |
        | send query 3   |                 |
        |                | process query 3 |
        | receive rows 3 |                 |
        ```
        Read more: https://docs.rs/tokio-postgres/latest/tokio_postgres/#pipelining

        ### Parameters:
        - `queries`: queries with parameters to execute.
        - `prepared`: should the querystring/querystrings be prepared before the request.
            By default any querystrings will be prepared.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()

            results: list[QueryResult] = await transaction.pipeline(
                queries=[
                    (
                        "SELECT username FROM users WHERE id = $1",
                        [100],
                    ),
                    (
                        "SELECT some_data FROM profiles",
                        None,
                    ),
                    (
                        "INSERT INTO users (username, id) VALUES ($1, $2)",
                        ["PSQLPy", 1],
                    ),
                ]
            )
        ```
        """

    async def create_savepoint(self: Self, savepoint_name: str) -> None:
        """Create new savepoint.

        One `savepoint_name` can be used once.


        If you specify the same savepoint name more than once
        exception will be raised.

        ### Parameters:
        - `savepoint_name`: name of the savepoint.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()

            await transaction.create_savepoint("my_savepoint")
            await transaction.execute(...)
            await transaction.rollback_savepoint("my_savepoint")
        ```
        """

    async def rollback_savepoint(self: Self, savepoint_name: str) -> None:
        """ROLLBACK to the specified `savepoint_name`.

        If you specified wrong savepoint name
        then exception will be raised.

        ### Parameters:
        - `savepoint_name`: name of the SAVEPOINT.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()

            await transaction.savepoint("my_savepoint")
            await transaction.execute(...)
            await transaction.rollback_savepoint("my_savepoint")
        ```
        """

    async def release_savepoint(self: Self, savepoint_name: str) -> None:
        """Execute ROLLBACK TO SAVEPOINT.

        If you specified wrong savepoint name
        then exception will be raised.

        ### Parameters:
        - `savepoint_name`: name of the SAVEPOINT.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = connection.transaction()

            await transaction.savepoint("my_savepoint")
            await transaction.release_savepoint
        ```
        """

    def cursor(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        fetch_number: int | None = None,
    ) -> Cursor:
        """Create new cursor object.

        Cursor can be used as an asynchronous iterator.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `fetch_number`: how many rows need to fetch.

        ### Returns:
        new initialized cursor.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            transaction = await connection.transaction()

            cursor = transaction.cursor(
                querystring="SELECT * FROM users WHERE username = $1",
                parameters=["Some_Username"],
                fetch_number=5,
            )
            await cursor.start()

            async for fetched_result in cursor:
                dict_result: List[Dict[Any, Any]] = fetched_result.result()
                ...  # do something with this result.

            await cursor.close()
        ```
        """

    async def binary_copy_to_table(
        self: Self,
        source: bytes | bytearray | Buffer | BytesIO,
        table_name: str,
        columns: Sequence[str] | None = None,
        schema_name: str | None = None,
    ) -> int:
        """Perform binary copy to PostgreSQL.

        Execute `COPY table_name (<columns>) FROM STDIN (FORMAT binary)`
        and start sending bytes to PostgreSQL.

        IMPORTANT! User is responsible for the bytes passed to the database.
        If bytes are incorrect user will get error from the database.

        ### Parameters:
        - `source`: source of bytes.
        - `table_name`: name of the table.
        - `columns`: sequence of str columns.
        - `schema_name`: name of the schema.

        ### Returns:
        number of inserted rows;
        """

async def connect(
    dsn: str | None = None,
    username: str | None = None,
    password: str | None = None,
    host: str | None = None,
    hosts: list[str] | None = None,
    port: int | None = None,
    ports: list[int] | None = None,
    db_name: str | None = None,
    target_session_attrs: TargetSessionAttrs | None = None,
    options: str | None = None,
    application_name: str | None = None,
    connect_timeout_sec: int | None = None,
    connect_timeout_nanosec: int | None = None,
    tcp_user_timeout_sec: int | None = None,
    tcp_user_timeout_nanosec: int | None = None,
    keepalives: bool | None = None,
    keepalives_idle_sec: int | None = None,
    keepalives_idle_nanosec: int | None = None,
    keepalives_interval_sec: int | None = None,
    keepalives_interval_nanosec: int | None = None,
    keepalives_retries: int | None = None,
    load_balance_hosts: LoadBalanceHosts | None = None,
    ssl_mode: SslMode | None = None,
    ca_file: str | None = None,
) -> Connection:
    """Create new standalone connection."""

class Connection:
    """Connection from Database Connection Pool.

    It can be created only from connection pool.
    """

    conn_dbname: str | None
    user: str | None
    host_addrs: list[str]
    hosts: list[str]
    ports: list[int]

    async def __aenter__(self: Self) -> Self: ...
    async def __aexit__(
        self: Self,
        exception_type: type[BaseException] | None,
        exception: BaseException | None,
        traceback: types.TracebackType | None,
    ) -> None: ...
    async def prepare(
        self,
        querystring: str,
        parameters: ParamsT = None,
    ) -> PreparedStatement:
        """Prepare statement.

        Return representation of prepared statement.
        """

    async def execute(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> QueryResult:
        """Execute the query.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Returns:
        query result as `QueryResult`

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            query_result: QueryResult = await connection.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
        ```
        """

    async def execute_batch(
        self: Self,
        querystring: str,
    ) -> None:
        """
        Execute a sequence of SQL statements using the simple query protocol.

        Statements should be separated by semicolons.
        If an error occurs, execution of the sequence will stop at that point.
        This is intended for use when, for example,
        initializing a database schema.

        ### Parameters:
        - `querystring`: querystrings separated by semicolons.
        """

    async def execute_many(
        self: Self,
        querystring: str,
        parameters: list[list[Any]] | None = None,
        prepared: bool = True,
    ) -> None: ...
    """Execute query multiple times with different parameters.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            await connection.execute_many(
                "INSERT INTO users (name, age) VALUES ($1, $2)",
                [["boba", 10], ["boba", 20]],
            )
        ```
        """
    async def fetch(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> QueryResult:
        """Fetch the result from database.

        It's the same as `execute` method, we made it because people are used
        to `fetch` method name.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.
        """

    async def fetch_row(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> SingleQueryResult:
        """Fetch exaclty single row from query.

        Query must return exactly one row, otherwise error will be raised.
        Querystring can contain `$<number>` parameters
        for converting them in the driver side.


        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()

            connection = await db_pool.connection()
            query_result: SingleQueryResult = await transaction.fetch_row(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: Dict[Any, Any] = query_result.result()
        ```
        """

    async def fetch_val(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        prepared: bool = True,
    ) -> Any:
        """Execute the query and return first value of the first row.

        Returns an error if the query does not return exactly one row.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `prepared`: should the querystring be prepared before the request.
            By default any querystring will be prepared.

        ### Raises
        - `RustPSQLDriverPyBaseError`: if the query does not
        return exactly one row

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            # this will be an int value
            query_result_value = await connection.fetch_row(
                "SELECT COUNT(*) FROM users WHERE id > $1",
                [100],
            )
        ```
        """

    def transaction(
        self,
        isolation_level: IsolationLevel | None = None,
        read_variant: ReadVariant | None = None,
        deferrable: bool | None = None,
    ) -> Transaction:
        """Create new transaction.

        ### Parameters:
        - `isolation_level`: configure isolation level of the transaction.
        - `read_variant`: configure read variant of the transaction.
        - `deferrable`: configure deferrable of the transaction.
        """

    def cursor(
        self: Self,
        querystring: str,
        parameters: ParamsT = None,
        fetch_number: int | None = None,
    ) -> Cursor:
        """Create new cursor object.

        Cursor can be used as an asynchronous iterator.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `fetch_number`: how many rows need to fetch.

        ### Returns:
        new initialized cursor.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            connection = await db_pool.connection()
            async with connection.transaction():
                async with connection.cursor(
                    querystring="SELECT * FROM users WHERE username = $1",
                    parameters=["Some_Username"],
                    fetch_number=5,
                ) as cursor:
                    async for fetched_result in cursor:
                        dict_result: List[Dict[Any, Any]] = fetched_result.result()
                        ...  # do something with this result.
        ```
        """

    def close(self: Self) -> None:
        """Return connection back to the pool.

        It necessary to commit all transactions and close all cursor
        made by this connection. Otherwise, it won't have any practical usage.
        """

    async def binary_copy_to_table(
        self: Self,
        source: bytes | bytearray | Buffer | BytesIO,
        table_name: str,
        columns: Sequence[str] | None = None,
        schema_name: str | None = None,
    ) -> int:
        """Perform binary copy to PostgreSQL.

        Execute `COPY table_name (<columns>) FROM STDIN (FORMAT binary)`
        and start sending bytes to PostgreSQL.

        IMPORTANT! User is responsible for the bytes passed to the database.
        If bytes are incorrect user will get error from the database.

        ### Parameters:
        - `source`: source of bytes.
        - `table_name`: name of the table.
        - `columns`: sequence of str columns.
        - `schema_name`: name of the schema.

        ### Returns:
        number of inserted rows;
        """

class ConnectionPoolStatus:
    max_size: int
    size: int
    available: int
    waiting: int

class ConnectionPool:
    """Connection pool for executing queries.

    This is the main entrypoint in the library.
    """

    def __init__(
        self: Self,
        dsn: str | None = None,
        username: str | None = None,
        password: str | None = None,
        host: str | None = None,
        hosts: list[str] | None = None,
        port: int | None = None,
        ports: list[int] | None = None,
        db_name: str | None = None,
        target_session_attrs: TargetSessionAttrs | None = None,
        options: str | None = None,
        application_name: str | None = None,
        connect_timeout_sec: int | None = None,
        connect_timeout_nanosec: int | None = None,
        tcp_user_timeout_sec: int | None = None,
        tcp_user_timeout_nanosec: int | None = None,
        keepalives: bool | None = None,
        keepalives_idle_sec: int | None = None,
        keepalives_idle_nanosec: int | None = None,
        keepalives_interval_sec: int | None = None,
        keepalives_interval_nanosec: int | None = None,
        keepalives_retries: int | None = None,
        load_balance_hosts: LoadBalanceHosts | None = None,
        max_db_pool_size: int = 2,
        conn_recycling_method: ConnRecyclingMethod | None = None,
        ssl_mode: SslMode | None = None,
        ca_file: str | None = None,
    ) -> None:
        """Create new PostgreSQL connection pool.

        It connects to the database and create pool.

        You cannot set the minimum size for the connection
        pool, by it is 0.
        `ConnectionPool` doesn't create connections on startup.
        It makes new connection on demand.

        If you specify `dsn` parameter then `username`, `password`,
        `host`, `hosts`, `port`, `ports`, `db_name` and `target_session_attrs`
        parameters will be ignored.

        ### Parameters:
        - `dsn`: Full dsn connection string.
            `postgres://postgres:postgres@localhost:5432/postgres?target_session_attrs=read-write`
        - `username`: Username of the user in the PostgreSQL
        - `password`: Password of the user in the PostgreSQL
        - `host`: Host of the PostgreSQL
        - `hosts`: Hosts of the PostgreSQL
        - `port`: Port of the PostgreSQL
        - `ports`: Ports of the PostgreSQL
        - `db_name`: Name of the database in PostgreSQL
        - `target_session_attrs`: Specifies requirements of the session.
        - `options`: Command line options used to configure the server
        - `application_name`: Sets the application_name parameter on the server.
        - `connect_timeout_sec`: The time limit in seconds applied to each socket-level
            connection attempt.
            Note that hostnames can resolve to multiple IP addresses,
            and this limit is applied to each address. Defaults to no timeout.
        - `connect_timeout_nanosec`: nanosec for connection timeout,
            can be used only with connect_timeout_sec.
        - `tcp_user_timeout_sec`: The time limit that
            transmitted data may remain unacknowledged
            before a connection is forcibly closed.
            This is ignored for Unix domain socket connections.
            It is only supported on systems where TCP_USER_TIMEOUT
            is available and will default to the system default if omitted
            or set to 0; on other systems, it has no effect.
        - `tcp_user_timeout_nanosec`: nanosec for cp_user_timeout,
            can be used only with tcp_user_timeout_sec.
        - `keepalives`: Controls the use of TCP keepalive.
            This option is ignored when connecting with Unix sockets.
            Defaults to on.
        - `keepalives_idle_sec`: The number of seconds of inactivity after
            which a keepalive message is sent to the server.
            This option is ignored when connecting with Unix sockets.
            Defaults to 2 hours.
        - `keepalives_idle_nanosec`: Nanosec for keepalives_idle_sec.
        - `keepalives_interval_sec`: The time interval between TCP keepalive probes.
            This option is ignored when connecting with Unix sockets.
        - `keepalives_interval_nanosec`: Nanosec for keepalives_interval_sec.
        - `keepalives_retries`: The maximum number of TCP keepalive probes
            that will be sent before dropping a connection.
            This option is ignored when connecting with Unix sockets.
        - `load_balance_hosts`: Controls the order in which the client tries to connect
            to the available hosts and addresses.
            Once a connection attempt is successful no other
            hosts and addresses will be tried.
            This parameter is typically used in combination with multiple host names
            or a DNS record that returns multiple IPs.
            If set to disable, hosts and addresses will be tried in the order provided.
            If set to random, hosts will be tried in a random order, and the IP addresses
            resolved from a hostname will also be tried in a random order.
            Defaults to disable.
        - `max_db_pool_size`: maximum size of the connection pool.
        - `conn_recycling_method`: how a connection is recycled.
        - `ssl_mode`: mode for ssl.
        - `ca_file`: Loads trusted root certificates from a file.
            The file should contain a sequence of PEM-formatted CA certificates.
        """

    def __iter__(self: Self) -> Self: ...
    def __enter__(self: Self) -> Self: ...
    def __exit__(
        self: Self,
        exception_type: type[BaseException] | None,
        exception: BaseException | None,
        traceback: types.TracebackType | None,
    ) -> None: ...
    def status(self: Self) -> ConnectionPoolStatus:
        """Return information about connection pool.

        ### Returns
        `ConnectionPoolStatus`
        """

    def resize(self: Self, new_max_size: int) -> None:
        """Resize the connection pool.

        This change the max_size of the pool dropping
        excess objects and/or making space for new ones.

        ### Parameters:
        - `new_max_size`: new size for the connection pool.
        """

    async def connection(self: Self) -> Connection:
        """Create new connection.

        It acquires new connection from the database pool.
        """

    def acquire(self: Self) -> Connection:
        """Create new connection for async context manager.

        Must be used only in async context manager.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult

        async def main() -> None:
            db_pool = PSQLPool()
            async with db_pool.acquire() as connection:
                res = await connection.execute(...)
        ```
        """

    def listener(self: Self) -> Listener:
        """Create new listener."""

    def close(self: Self) -> None:
        """Close the connection pool."""

def connect_pool(
    dsn: str | None = None,
    username: str | None = None,
    password: str | None = None,
    host: str | None = None,
    hosts: list[str] | None = None,
    port: int | None = None,
    ports: list[int] | None = None,
    db_name: str | None = None,
    target_session_attrs: TargetSessionAttrs | None = None,
    options: str | None = None,
    application_name: str | None = None,
    connect_timeout_sec: int | None = None,
    connect_timeout_nanosec: int | None = None,
    tcp_user_timeout_sec: int | None = None,
    tcp_user_timeout_nanosec: int | None = None,
    keepalives: bool | None = None,
    keepalives_idle_sec: int | None = None,
    keepalives_idle_nanosec: int | None = None,
    keepalives_interval_sec: int | None = None,
    keepalives_interval_nanosec: int | None = None,
    keepalives_retries: int | None = None,
    load_balance_hosts: LoadBalanceHosts | None = None,
    max_db_pool_size: int = 2,
    conn_recycling_method: ConnRecyclingMethod | None = None,
    ssl_mode: SslMode | None = None,
    ca_file: str | None = None,
) -> ConnectionPool:
    """Create new PostgreSQL connection pool.

    It connects to the database and create pool.

    You cannot set the minimum size for the connection
    pool, by it is 0.
    `ConnectionPool` doesn't create connections on startup.
    It makes new connection on demand.

    If you specify `dsn` parameter then `username`, `password`,
    `host`, `hosts`, `port`, `ports`, `db_name` and `target_session_attrs`
    parameters will be ignored.

    ### Parameters:
    - `dsn`: Full dsn connection string.
        `postgres://postgres:postgres@localhost:5432/postgres?target_session_attrs=read-write`
    - `username`: Username of the user in the PostgreSQL
    - `password`: Password of the user in the PostgreSQL
    - `host`: Host of the PostgreSQL
    - `hosts`: Hosts of the PostgreSQL
    - `port`: Port of the PostgreSQL
    - `ports`: Ports of the PostgreSQL
    - `db_name`: Name of the database in PostgreSQL
    - `target_session_attrs`: Specifies requirements of the session.
    - `options`: Command line options used to configure the server
    - `application_name`: Sets the application_name parameter on the server.
    - `connect_timeout_sec`: The time limit in seconds applied to each socket-level
        connection attempt.
        Note that hostnames can resolve to multiple IP addresses,
        and this limit is applied to each address. Defaults to no timeout.
    - `connect_timeout_nanosec`: nanosec for connection timeout,
        can be used only with connect_timeout_sec.
    - `tcp_user_timeout_sec`: The time limit that
        transmitted data may remain unacknowledged
        before a connection is forcibly closed.
        This is ignored for Unix domain socket connections.
        It is only supported on systems where TCP_USER_TIMEOUT
        is available and will default to the system default if omitted
        or set to 0; on other systems, it has no effect.
    - `tcp_user_timeout_nanosec`: nanosec for cp_user_timeout,
        can be used only with tcp_user_timeout_sec.
    - `keepalives`: Controls the use of TCP keepalive.
        This option is ignored when connecting with Unix sockets.
        Defaults to on.
    - `keepalives_idle_sec`: The number of seconds of inactivity after
        which a keepalive message is sent to the server.
        This option is ignored when connecting with Unix sockets.
        Defaults to 2 hours.
    - `keepalives_idle_nanosec`: Nanosec for keepalives_idle_sec.
    - `keepalives_interval_sec`: The time interval between TCP keepalive probes.
        This option is ignored when connecting with Unix sockets.
    - `keepalives_interval_nanosec`: Nanosec for keepalives_interval_sec.
    - `keepalives_retries`: The maximum number of TCP keepalive probes
        that will be sent before dropping a connection.
        This option is ignored when connecting with Unix sockets.
    - `load_balance_hosts`: Controls the order in which the client tries to connect
        to the available hosts and addresses.
        Once a connection attempt is successful no other
        hosts and addresses will be tried.
        This parameter is typically used in combination with multiple host names
        or a DNS record that returns multiple IPs.
        If set to disable, hosts and addresses will be tried in the order provided.
        If set to random, hosts will be tried in a random order, and the IP addresses
        resolved from a hostname will also be tried in a random order.
        Defaults to disable.
    - `max_db_pool_size`: maximum size of the connection pool.
    - `conn_recycling_method`: how a connection is recycled.
    - `ssl_mode`: mode for ssl.
    - `ca_file`: Loads trusted root certificates from a file.
        The file should contain a sequence of PEM-formatted CA certificates.
    """

class ConnectionPoolBuilder:
    """Builder for `ConnectionPool`."""

    def __init__(self: Self) -> None:
        """Initialize new instance of `ConnectionPoolBuilder`."""

    def build(self: Self) -> ConnectionPool:
        """
        Build `ConnectionPool`.

        ### Returns:
        `ConnectionPool`
        """

    def max_pool_size(self: Self, pool_size: int) -> Self:
        """
        Set maximum connection pool size.

        ### Parameters:
        - `pool_size`: size of the pool, must be more than 1.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def conn_recycling_method(
        self: Self,
        conn_recycling_method: ConnRecyclingMethod,
    ) -> Self:
        """
        Set connection recycling method.

        Connection recycling method is how a connection is recycled.

        ### Parameters:
        - `conn_recycling_method`: ConnRecyclingMethod enum.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def user(self: Self, user: str) -> Self:
        """
        Set username to `PostgreSQL`.

        ### Parameters:
        - `user`: username of the PostgreSQL user.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def password(self: Self, password: str) -> Self:
        """
        Set password for `PostgreSQL`.

        ### Parameters:
        - `password`: password for the `PostgreSQL` user.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def dbname(self: Self, dbname: str) -> Self:
        """
        Set database name for the `PostgreSQL`.

        ### Parameters:
        - `dbname`: database for the `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def options(self: Self, options: str) -> Self:
        """
        Set command line options used to configure the server.

        ### Parameters:
        - `options`: command line options

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def application_name(self: Self, application_name: str) -> Self:
        """
        Set the value of the `application_name` runtime parameter.

        ### Parameters:
        - `application_name`: `application_name` runtime parameter

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def ssl_mode(self: Self, ssl_mode: SslMode) -> Self:
        """
        Set the SSL configuration.

        ### Parameters:
        - `ssl_mode`: mode for TLS.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def ca_file(self: Self, ca_file: str) -> Self:
        """
        Set ca_file for SSL.

        ### Parameters:
        - `ca_file`: certificate file to connection to PostgreSQL.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def host(self: Self, host: str) -> Self:
        """
        Add a host to the configuration.

        Multiple hosts can be specified by calling this method multiple times,
        and each will be tried in order.
        On Unix systems, a host starting with a `/` is interpreted
        as a path to a directory containing Unix domain sockets.
        There must be either no hosts,
        or the same number of hosts as hostaddrs.

        ### Parameters:
        - `host`: host to `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def hostaddr(self: Self, hostaddr: IPv4Address | IPv6Address) -> Self:
        """
        Add a hostaddr to the configuration.

        Multiple hostaddrs can be specified by calling
        this method multiple times, and each will be tried in order.
        There must be either no hostaddrs,
        or the same number of hostaddrs as hosts.

        ### Parameters:
        - `hostaddr`: hostaddr to `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def port(self: Self, port: int) -> Self:
        """
        Add a port to the configuration.

        Multiple ports can be specified by calling this method multiple times.
        There must either be no ports,
        in which case the default of 5432 is used,
        a single port, in which it is used for all hosts,
        or the same number of ports as hosts.

        ### Parameters:
        - `port`: port for hosts to `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def connect_timeout(self: Self, connect_timeout: int) -> Self:
        """
        Set the timeout applied to socket-level connection attempts.

        Note that hostnames can resolve to multiple IP addresses,
        and this timeout will apply to each address of each
        host separately. Defaults to no limit.

        ### Parameters:
        - `connect_timeout`: connection timeout to `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def tcp_user_timeout(self: Self, tcp_user_timeout: int) -> Self:
        """
        Set the TCP user timeout.

        This is ignored for Unix domain socket connections.
        It is only supported on systems where TCP_USER_TIMEOUT is available
        and will default to the system default if omitted or set to 0;
        on other systems, it has no effect.

        ### Parameters:
        - `tcp_user_timeout`: tcp_user_timeout to `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def target_session_attrs(
        self: Self,
        target_session_attrs: TargetSessionAttrs,
    ) -> Self:
        """
        Set the requirements of the session.

        This can be used to connect to the primary server in a
        clustered database rather than one of the read-only
        secondary servers. Defaults to `Any`.

        ### Parameters:
        - `target_session_attrs`: target_session_attrs for `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def load_balance_hosts(
        self: Self,
        load_balance_hosts: LoadBalanceHosts,
    ) -> Self:
        """
        Set the host load balancing behavior.

        Defaults to `disable`.

        ### Parameters:
        - `load_balance_hosts`: load_balance_hosts for `PostgreSQL`.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def keepalives(
        self: Self,
        keepalives: bool,
    ) -> Self:
        """
        Control the use of TCP keepalive.

        This is ignored for Unix domain socket connections.

        Defaults to `true`.

        ### Parameters:
        - `keepalives`: boolean value for keepalives.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def keepalives_idle(
        self: Self,
        keepalives_idle: int,
    ) -> Self:
        """
        Set the amount of idle time before a keepalive packet is sent on the connection.

        This is ignored for Unix domain sockets,
        or if the `keepalives` option is disabled.

        Defaults to 2 hours.

        ### Parameters:
        - `keepalives_idle`: number in secs for idle.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def keepalives_interval(
        self: Self,
        keepalives_interval: int,
    ) -> Self:
        """
        Set the time interval between TCP keepalive probes.

        On Windows, this sets the value of the
        tcp_keepalive struct keepalive interval field.

        This is ignored for Unix domain sockets,
        or if the `keepalives` option is disabled.

        ### Parameters:
        - `keepalives_interval`: number in secs for interval.

        ### Returns:
        `ConnectionPoolBuilder`
        """

    def keepalives_retries(
        self: Self,
        keepalives_retries: int,
    ) -> Self:
        """Keepalives Retries.

        Set the maximum number of TCP keepalive probes
        that will be sent before dropping a connection.

        This is ignored for Unix domain sockets,
        or if the `keepalives` option is disabled.

        ### Parameters:
        - `keepalives_retries`: number of retries.

        ### Returns:
        `ConnectionPoolBuilder`
        """

class Listener:
    """Listener for LISTEN command.

    Can be used two ways:
    1) As a background task
    2) As an asynchronous iterator

    ## Examples

    ### Background task:

    ```python
    async def callback(
        channel: str,
        payload: str,
        process_id: int,
        connection: Connection,
    ) -> None: ...
    async def main():
        pool = ConnectionPool()

        listener = pool.listener()
        await listener.add_callback(
            channel="test_channel",
            callback=callback,
        )
        await listener.startup()

        listener.listen()
    ```

    ### Async iterator
    ```python
    from psqlpy import

    async def msg_processor(
        msg: ListenerNotificationMsg,
    ) -> None:
        ...


    async def main():
        pool = ConnectionPool()

        listener = pool.listener()
        await listener.add_callback(
            channel="test_channel",
            callback=callback,
        )
        await listener.startup()

        for msg in listener:
            await msg_processor(msg)
    ```
    """

    connection: Connection
    is_started: bool

    def __aiter__(self: Self) -> Self: ...
    async def __anext__(self: Self) -> ListenerNotificationMsg: ...
    async def __aenter__(self: Self) -> Self: ...
    async def __aexit__(
        self: Self,
        exception_type: type[BaseException] | None,
        exception: BaseException | None,
        traceback: types.TracebackType | None,
    ) -> None: ...
    async def startup(self: Self) -> None:
        """Startup the listener.

        Each listener MUST be started up.
        """

    async def shutdown(self: Self) -> None:
        """Shutdown the listener.

        Abort listen and release underlying connection.
        """

    async def add_callback(
        self: Self,
        channel: str,
        callback: Callable[
            [Connection, str, str, int],
            Awaitable[None],
        ],
    ) -> None:
        """Add callback to the channel.

        Callback must be async function and have signature like this:
        ```python
        async def callback(
            connection: Connection,
            payload: str,
            channel: str,
            process_id: int,
        ) -> None: ...
        ```

        Callback parameters are passed as args on the Rust side.
        """

    async def clear_channel_callbacks(self, channel: str) -> None:
        """Remove all callbacks for the channel.

        ### Parameters:
        - `channel`: name of the channel.
        """

    async def clear_all_channels(self) -> None:
        """Clear all channels callbacks."""

    def listen(self: Self) -> None:
        """Start listening.

        Start actual listening.
        In the background it creates task in Rust event loop.
        """

    def abort_listen(self: Self) -> None:
        """Abort listen.

        If `listen()` method was called, stop listening,
        else don't do anything.
        """

class ListenerNotificationMsg:
    """Listener message in async iterator."""

    process_id: int
    channel: str
    payload: str
    connection: Connection

class Column:
    name: str
    table_oid: int | None

class PreparedStatement:
    async def execute(self: Self) -> QueryResult:
        """Execute prepared statement."""

    def cursor(self: Self) -> Cursor:
        """Create new server-side cursor based on prepared statement."""

    def columns(self: Self) -> list[Column]:
        """Return information about statement columns."""
