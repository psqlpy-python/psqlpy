import types
from enum import Enum
from typing import Any, Callable, Optional, TypeVar

from typing_extensions import Self

_CustomClass = TypeVar(
    "_CustomClass",
)

class QueryResult:
    """Result."""

    def result(self: Self) -> list[dict[Any, Any]]:
        """Return result from database as a list of dicts."""
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
            await db_pool.startup()

            query_result: QueryResult = await db_pool.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            class_results: List[ExampleOfAsClass] = query_result.as_class(
                as_class=ExampleOfAsClass,
            )
        ```
        """

class SingleQueryResult:
    """Single result."""

    def result(self: Self) -> dict[Any, Any]:
        """Return result from database as a dict."""
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
            await db_pool.startup()

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

class IsolationLevel(Enum):
    """Class for Isolation Level for transactions."""

    ReadUncommitted = 1
    ReadCommitted = 2
    RepeatableRead = 3
    Serializable = 4

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

class Cursor:
    """Represent opened cursor in a transaction.

    It can be used as an asynchronous iterator.
    """

    async def fetch(
        self: Self,
        fetch_number: int | None = None,
    ) -> QueryResult:
        """Fetch next <fetch_number> rows.

        By default fetches 10 next rows.

        ### Parameters:
        - `fetch_number`: how many rows need to fetch.

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_next(
        self: Self,
    ) -> QueryResult:
        """Fetch next row.

        Execute FETCH NEXT

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_prior(
        self: Self,
    ) -> QueryResult:
        """Fetch previous row.

        Execute FETCH PRIOR

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_first(
        self: Self,
    ) -> QueryResult:
        """Fetch first row.

        Execute FETCH FIRST

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_last(
        self: Self,
    ) -> QueryResult:
        """Fetch last row.

        Execute FETCH LAST

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_absolute(
        self: Self,
        absolute_number: int,
    ) -> QueryResult:
        """Fetch absolute rows.

        Execute FETCH ABSOLUTE <absolute_number>.

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_relative(
        self: Self,
        relative_number: int,
    ) -> QueryResult:
        """Fetch absolute rows.

        Execute FETCH RELATIVE <relative_number>.

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_forward_all(
        self: Self,
    ) -> QueryResult:
        """Fetch forward all rows.

        Execute FETCH FORWARD ALL.

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_backward(
        self: Self,
        backward_count: int,
    ) -> QueryResult:
        """Fetch backward rows.

        Execute FETCH BACKWARD <backward_count>.

        ### Returns:
        result as `QueryResult`.
        """
    async def fetch_backward_all(
        self: Self,
    ) -> QueryResult:
        """Fetch backward all rows.

        Execute FETCH BACKWARD ALL.

        ### Returns:
        result as `QueryResult`.
        """
    async def close(self: Self) -> None:
        """Close the cursor.

        Execute CLOSE command for the cursor.
        """
    def __aiter__(self: Self) -> Self: ...
    async def __anext__(self: Self) -> QueryResult: ...

class Transaction:
    """Single connection for executing queries.

    It represents transaction in database.

    You can create it only from `PSQLPool` with method
    `.transaction()`.
    """

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
    async def execute(
        self: Self,
        querystring: str,
        parameters: list[Any] | None = None,
    ) -> QueryResult:
        """Execute the query.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            transaction = await db_pool.transaction()
            await transaction.begin()
            query_result: QueryResult = await transaction.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
            # You must call commit manually
            await transaction.commit()

        # Or you can transaction as a async context manager

        async def main() -> None:
            db_pool = PSQLPool()
            await psqlpy.startup()

            transaction = await db_pool.transaction()
            async with transaction:
                query_result: QueryResult = await transaction.execute(
                    "SELECT username FROM users WHERE id = $1",
                    [100],
                )
                dict_result: List[Dict[Any, Any]] = query_result.result()
            # This way transaction begins and commits by itself.
        ```
        """
    async def execute_many(
        self: Self,
        querystring: str,
        parameters: list[list[Any]] | None = None,
    ) -> None: ...
    """Execute query multiple times with different parameters.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of list of parameters to pass in the query.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            transaction = await db_pool.transaction()
            await transaction.begin()
            query_result: QueryResult = await transaction.execute_many(
                "INSERT INTO users (name, age) VALUES ($1, $2)",
                [["boba", 10], ["boba", 20]],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
            # You must call commit manually
            await transaction.commit()

        # Or you can transaction as a async context manager

        async def main() -> None:
            db_pool = PSQLPool()
            await psqlpy.startup()

            transaction = await db_pool.transaction()
            async with transaction:
                query_result: QueryResult = await transaction.execute(
                    "SELECT username FROM users WHERE id = $1",
                    [100],
                )
                dict_result: List[Dict[Any, Any]] = query_result.result()
            # This way transaction begins and commits by itself.
        ```
        """
    async def fetch_row(
        self: Self,
        querystring: str,
        parameters: list[Any] | None = None,
    ) -> SingleQueryResult:
        """Fetch exaclty single row from query.

        Query must return exactly one row, otherwise error will be raised.
        Querystring can contain `$<number>` parameters
        for converting them in the driver side.


        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            transaction = await db_pool.transaction()
            await transaction.begin()
            query_result: SingleQueryResult = await transaction.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: Dict[Any, Any] = query_result.result()
            # You must call commit manually
            await transaction.commit()

        # Or you can transaction as a async context manager

        async def main() -> None:
            db_pool = PSQLPool()
            await psqlpy.startup()

            transaction = await db_pool.transaction()
            async with transaction:
                query_result: SingleQueryResult = await transaction.execute(
                    "SELECT username FROM users WHERE id = $1 LIMIT 1",
                    [100],
                )
                dict_result: Dict[Any, Any] = query_result.result()
            # This way transaction begins and commits by itself.
        ```
        """
    async def fetch_val(
        self: Self,
        querystring: str,
        parameters: list[Any] | None = None,
    ) -> Any | None:
        """Execute the query and return first value of the first row.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            transaction = await db_pool.transaction()
            await transaction.begin()
            value: Any | None = await transaction.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )

        # Or you can transaction as a async context manager

        async def main() -> None:
            db_pool = PSQLPool()
            await psqlpy.startup()

            transaction = await db_pool.transaction()
            async with transaction:
                query_result: SingleQueryResult = await transaction.execute(
                    "SELECT username FROM users WHERE id = $1",
                    [100],
                )
                dict_result: Dict[Any, Any] = query_result.result()
            # This way transaction begins and commits by itself.
        ```
        """
    async def pipeline(
        self,
        queries: list[tuple[str, list[Any] | None]],
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
        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            transaction = await db_pool.transaction()

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
        """  # noqa: E501
    async def savepoint(self: Self, savepoint_name: str) -> None:
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
            await db_pool.startup()

            transaction = await db_pool.transaction()

            await transaction.savepoint("my_savepoint")
            await transaction.execute(...)
            await transaction.rollback_to("my_savepoint")
        ```
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
            await db_pool.startup()

            transaction = await db_pool.transaction()
            await transaction.execute(...)
            await transaction.rollback()
        ```
        """
    async def rollback_to(self: Self, savepoint_name: str) -> None:
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
            await db_pool.startup()

            transaction = await db_pool.transaction()

            await transaction.savepoint("my_savepoint")
            await transaction.execute(...)
            await transaction.rollback_to("my_savepoint")
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
            await db_pool.startup()

            transaction = await db_pool.transaction()

            await transaction.savepoint("my_savepoint")
            await transaction.release_savepoint
        ```
        """
    async def cursor(
        self: Self,
        querystring: str,
        parameters: list[Any] | None = None,
        fetch_number: int | None = None,
        scroll: bool | None = None,
    ) -> Cursor:
        """Create new cursor object.

        Cursor can be used as an asynchronous iterator.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.
        - `fetch_number`: how many rows need to fetch.
        - `scroll`: SCROLL or NO SCROLL cursor.

        ### Returns:
        new initialized cursor.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            connection = await db_pool.connection()
            transaction = await connection.transaction()

            cursor = await transaction.cursor(
                querystring="SELECT * FROM users WHERE username = $1",
                parameters=["Some_Username"],
                fetch_number=5,
            )

            async for fetched_result in cursor:
                dict_result: List[Dict[Any, Any]] = fetched_result.result()
                ... # do something with this result.
        ```
        """

class Connection:
    """Connection from Database Connection Pool.

    It can be created only from connection pool.
    """

    async def execute(
        self: Self,
        querystring: str,
        parameters: list[Any] | None = None,
    ) -> QueryResult:
        """Execute the query.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.

        ### Returns:
        query result as `QueryResult`

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            connection = await db_pool.connection()
            query_result: QueryResult = await connection.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
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
        """

class PSQLPool:
    """Connection pool for executing queries.

    This is the main entrypoint in the library.
    """

    def __init__(
        self: Self,
        dsn: Optional[str] = None,
        username: Optional[str] = None,
        password: Optional[str] = None,
        host: Optional[str] = None,
        port: Optional[int] = None,
        db_name: Optional[str] = None,
        max_db_pool_size: Optional[str] = None,
        conn_recycling_method: Optional[ConnRecyclingMethod] = None,
    ) -> None:
        """Create new PostgreSQL connection pool.

        It connects to the database and create pool.

        You cannot set the minimum size for the connection
        pool, by default it is 1.

        This connection pool can:
        - Startup itself with `startup` method
        - Execute queries and return `QueryResult` class as a result
        - Create new instance of `Transaction`

        ### Parameters:
        - `dsn`: full dsn connection string.
            `postgres://postgres:postgres@localhost:5432/postgres?target_session_attrs=read-write`
        - `username`: username of the user in postgres
        - `password`: password of the user in postgres
        - `host`: host of postgres
        - `port`: port of postgres
        - `db_name`: name of the database in postgres
        - `max_db_pool_size`: maximum size of the connection pool
        - `conn_recycling_method`: how a connection is recycled.
        """
    async def startup(self: Self) -> None:
        """Startup the connection pool.

        You must call it before start making queries.
        """
    async def execute(
        self: Self,
        querystring: str,
        parameters: list[Any] | None = None,
    ) -> QueryResult:
        """Execute the query.

        Querystring can contain `$<number>` parameters
        for converting them in the driver side.

        ### Parameters:
        - `querystring`: querystring to execute.
        - `parameters`: list of parameters to pass in the query.

        ### Example:
        ```python
        import asyncio

        from psqlpy import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await psqlpy.startup()
            query_result: QueryResult = await psqlpy.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
            # you don't need to close the pool,
            # it will be dropped on Rust side.
        ```
        """
    async def connection(self: Self) -> Connection:
        """Create new connection.

        It acquires new connection from the database pool.
        """
