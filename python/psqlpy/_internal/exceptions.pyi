class RustPSQLDriverPyBaseError(Exception):
    """Base PSQL-Rust-Engine exception."""

class BaseConnectionPoolError(RustPSQLDriverPyBaseError):
    """Base error for all Connection Pool errors."""

class ConnectionPoolBuildError(BaseConnectionPoolError):
    """Error for errors in building connection pool."""

class ConnectionPoolConfigurationError(BaseConnectionPoolError):
    """Error in connection pool configuration."""

class ConnectionPoolExecuteError(BaseConnectionPoolError):
    """Error in connection pool execution."""

class BaseConnectionError(RustPSQLDriverPyBaseError):
    """Base error for Connection errors."""

class ConnectionExecuteError(BaseConnectionError):
    """Error in connection execution."""

class ConnectionClosedError(BaseConnectionError):
    """Error if underlying connection is already closed."""

class BaseTransactionError(RustPSQLDriverPyBaseError):
    """Base error for all transaction errors."""

class TransactionBeginError(BaseTransactionError):
    """Error in transaction begin."""

class TransactionCommitError(BaseTransactionError):
    """Error in transaction commit."""

class TransactionRollbackError(BaseTransactionError):
    """Error in transaction rollback."""

class TransactionSavepointError(BaseTransactionError):
    """Error in transaction savepoint."""

class TransactionExecuteError(BaseTransactionError):
    """Error in transaction execution."""

class TransactionClosedError(BaseTransactionError):
    """Error if underlying connection is already closed."""

class BaseCursorError(RustPSQLDriverPyBaseError):
    """Base error for Cursor errors."""

class CursorStartError(BaseCursorError):
    """Error in cursor declare."""

class CursorCloseError(BaseCursorError):
    """Error in cursor close."""

class CursorFetchError(BaseCursorError):
    """Error in cursor fetch (any fetch)."""

class CursorClosedError(BaseCursorError):
    """Error if underlying connection is already closed."""

class UUIDValueConvertError(RustPSQLDriverPyBaseError):
    """Error if it's impossible to convert py string UUID into rust UUID."""

class MacAddrConversionError(RustPSQLDriverPyBaseError):
    """Error if cannot convert MacAddr string value to rust type."""

class RustToPyValueMappingError(RustPSQLDriverPyBaseError):
    """Error if it is not possible to covert rust type to python.

    You can get it if you database contains data type that it not
    supported by this library.

    It's better to handle this exception.
    """

class PyToRustValueMappingError(RustPSQLDriverPyBaseError):
    """Error if it is not possible to covert python type to rust.

    You can get this exception when executing queries with parameters.
    So, if there are no parameters for the query, don't handle this error.
    """
