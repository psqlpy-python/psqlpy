from __future__ import annotations
from typing_extensions import Self

from typing import Dict, Optional, Any, List


class QueryResult:
    """Result."""

    def result(self: Self) -> List[Dict[Any, Any]]:
        """Return result from database as a list of dicts."""


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
        parameters: List[Any],
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


class PSQLPool:
    """Connection pool for executing queries.
    
    This is the main entrypoint in the library.
    """

    def __init__(
        self: Self,
        username: Optional[str],
        password: Optional[str],
        host: Optional[str],
        port: Optional[int],
        db_name: Optional[str],
        max_db_pool_size: Optional[str],
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
        parameters: List[Any],
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

    async def transaction(self) -> Transaction:
        """Create new transaction.
        
        It acquires new connection from the database pool
        and make it acts as transaction.
        """
