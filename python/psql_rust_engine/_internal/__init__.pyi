from typing import Dict, Optional, Any, List


class RustEnginePyQueryResult:
    """Result."""

    def __init__(
        self,
        inner: Any,
    ) -> None:
        ...
    def result(self) -> List[Dict[Any, Any]]:
        """"""


class RustEngine:
    """Rust engine."""

    def __init__(
        self,
        username: Optional[str],
        password: Optional[str],
        host: Optional[str],
        port: Optional[int],
        db_name: Optional[str],
    ) -> None:
        """Test ebana."""

    async def startup(self) -> None:
        ...
    
    async def execute(
        self,
        querystring: str,
        parameters: List[Any],
    ) -> RustEnginePyQueryResult:
        ...


class PyRustEngine:
    """Aboba"""

    def __init__(
        self,
        username: Optional[str],
        password: Optional[str],
        host: Optional[str],
        port: Optional[int],
        db_name: Optional[str],
    ) -> None:
        """Test ebana."""

    async def startup(self) -> None:
        ...
    
    async def execute(
        self,
        querystring: str,
        parameters: List[Any],
    ) -> RustEnginePyQueryResult:
        ...

    async def transaction(self):
        ...
