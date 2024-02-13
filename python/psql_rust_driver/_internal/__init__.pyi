from __future__ import annotations
from enum import Enum
from typing_extensions import Self

from typing import Dict, Optional, Any, List


class QueryResult:
    """Result."""

    def result(self: Self) -> List[Dict[Any, Any]]:
        """Return result from database as a list of dicts."""


class IsolationLevel(Enum):
    """Class for Isolation Level for transactions."""

    ReadUncommitted = 1
    ReadCommitted = 2
    RepeatableRead = 3
    Serializable = 4


class Transaction:
    """Single connection for executing queries.

    It represents transaction in database.

    You can create it only from `PSQLPool` with method
    `.transaction()`.
    """

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
        parameters: List[Any] | None = None,
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

        from psql_rust_driver import PSQLPool, QueryResult


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
            await psql_rust_driver.startup()

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

        from psql_rust_driver import PSQLPool, QueryResult


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

        from psql_rust_driver import PSQLPool, QueryResult


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

        from psql_rust_driver import PSQLPool, QueryResult


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

        from psql_rust_driver import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await db_pool.startup()

            transaction = await db_pool.transaction()

            await transaction.savepoint("my_savepoint")
            await transaction.release_savepoint
        ```
        """


class PSQLPool:
    """Connection pool for executing queries.
    
    This is the main entrypoint in the library.
    """

    def __init__(
        self: Self,
        username: Optional[str] = None,
        password: Optional[str] = None,
        host: Optional[str] = None,
        port: Optional[int] = None,
        db_name: Optional[str] = None,
        max_db_pool_size: Optional[str] = None,
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
        - `username`: username of the user in postgres
        - `password`: password of the user in postgres
        - `host`: host of postgres
        - `port`: port of postgres
        - `db_name`: name of the database in postgres
        - `max_db_pool_size`: maximum size of the connection pool
        """

    async def startup(self: Self) -> None:
        """Startup the connection pool.
        
        You must call it before start making queries.
        """
    
    async def execute(
        self: Self,
        querystring: str,
        parameters: List[Any] | None = None,
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

        from psql_rust_driver import PSQLPool, QueryResult


        async def main() -> None:
            db_pool = PSQLPool()
            await psql_rust_driver.startup()
            query_result: QueryResult = await psql_rust_driver.execute(
                "SELECT username FROM users WHERE id = $1",
                [100],
            )
            dict_result: List[Dict[Any, Any]] = query_result.result()
            # you don't need to close the pool, it will be dropped on Rust side.
        ```
        """
        ...

    async def transaction(
        self,
        isolation_level: IsolationLevel | None = IsolationLevel.ReadCommitted,
    ) -> Transaction:
        """Create new transaction.
        
        It acquires new connection from the database pool
        and make it acts as transaction.

        ### Parameters:
        - `isolation_level`: configure isolation level of the transaction.
        """
