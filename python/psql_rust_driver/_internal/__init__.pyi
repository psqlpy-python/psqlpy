from typing import Dict, Optional, Any, List


class QueryResult:
    """Result."""

    def __init__(
        self,
        inner: Any,
    ) -> None:
        ...
    def result(self) -> List[Dict[Any, Any]]:
        """"""


class PSQLPool:
    """Aboba"""

    def __init__(
        self,
        username: Optional[str],
        password: Optional[str],
        host: Optional[str],
        port: Optional[int],
        db_name: Optional[str],
        max_db_pool_size: Optional[str],
    ) -> None:
        """Test ebana."""

    async def startup(self) -> None:
        ...
    
    async def execute(
        self,
        querystring: str,
        parameters: List[Any],
    ) -> QueryResult:
        ...

    async def transaction(self):
        ...
