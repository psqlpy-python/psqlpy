from typing import Optional
from psqlpy._internal import Connection, ConnectionPool, QueryResult, Transaction, Cursor


# class Cursor(Cursor):
#     async def __anext__(self) -> Optional[QueryResult]:
#         results = await self.fetch(10)
#         if not results.result():
#             return StopAsyncIteration
#         return results


__all__ = [
    "ConnectionPool",
    "Transaction",
    "Connection",
    "Cursor",
]
