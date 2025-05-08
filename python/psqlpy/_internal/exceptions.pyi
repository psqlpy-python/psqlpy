class WarningError(Exception):
    """
    Exception raised for important warnings
    like data truncations while inserting, etc.
    """

class Error(Exception):
    """
    Exception that is the base class of all other error exceptions.

    You can use this to catch all errors with one single except statement.
    """

class InterfaceError(Error):
    """
    Exception raised for errors that are related to the
    database interface rather than the database itself.
    """

class DatabaseError(Error):
    """Exception raised for errors that are related to the database."""

class DataError(DatabaseError):
    """
    Exception raised for errors that are due to problems with
    the processed data like division by zero, numeric value out of range, etc.
    """

class OperationalError(DatabaseError):
    """
    Exception raised for errors that are related to the database’s operation
    and not necessarily under the control of the programmer,
    e.g. an unexpected disconnect occurs, the data source name is not found,
    a transaction could not be processed, a memory allocation error
    occurred during processing, etc.
    """

class IntegrityError(DatabaseError):
    """
    Exception raised when the relational integrity of the
    database is affected, e.g. a foreign key check fails.
    """

class InternalError(DatabaseError):
    """
    Exception raised when the database encounters an internal error,
    e.g. the cursor is not valid anymore, the transaction is out of sync, etc.
    """

class ProgrammingError(DatabaseError):
    """
    Exception raised for programming errors, e.g. table not found or
    already exists, syntax error in the SQL statement,
    wrong number of parameters specified, etc.
    """

class NotSupportedError(DatabaseError):
    """
    Exception raised in case a method or database API was used which
    is not supported by the database, e.g. requesting a .rollback()
    on a connection that does not support transaction
    or has transactions turned off.
    """

class BaseConnectionPoolError(InterfaceError):
    """Base error for all Connection Pool errors."""

class ConnectionPoolBuildError(BaseConnectionPoolError):
    """Error for errors in building connection pool."""

class ConnectionPoolConfigurationError(BaseConnectionPoolError):
    """Error in connection pool configuration."""

class ConnectionPoolExecuteError(BaseConnectionPoolError):
    """Error in connection pool execution."""

class BaseConnectionError(InterfaceError):
    """Base error for Connection errors."""

class ConnectionExecuteError(BaseConnectionError):
    """Error in connection execution."""

class ConnectionClosedError(BaseConnectionError):
    """Error if underlying connection is already closed."""

class BaseTransactionError(InterfaceError):
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

class BaseCursorError(InterfaceError):
    """Base error for Cursor errors."""

class CursorStartError(BaseCursorError):
    """Error in cursor declare."""

class CursorCloseError(BaseCursorError):
    """Error in cursor close."""

class CursorFetchError(BaseCursorError):
    """Error in cursor fetch (any fetch)."""

class CursorClosedError(BaseCursorError):
    """Error if underlying connection is already closed."""

class UUIDValueConvertError(DataError):
    """Error if it's impossible to convert py string UUID into rust UUID."""

class MacAddrConversionError(DataError):
    """Error if cannot convert MacAddr string value to rust type."""

class RustToPyValueMappingError(DataError):
    """Error if it is not possible to covert rust type to python.

    You can get it if you database contains data type that it not
    supported by this library.
    """

class PyToRustValueMappingError(DataError):
    """Error if it is not possible to covert python type to rust.

    You can get this exception when executing queries with parameters.
    So, if there are no parameters for the query, don't handle this error.
    """

class BaseListenerError(InterfaceError):
    """Base error for all Listener errors."""

class ListenerStartError(BaseListenerError):
    """Error if listener start failed."""

class ListenerClosedError(BaseListenerError):
    """Error if listener manipulated but it's closed."""

class ListenerCallbackError(BaseListenerError):
    """Error if callback passed to listener isn't a coroutine."""
