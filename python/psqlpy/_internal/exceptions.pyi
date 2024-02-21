class RustPSQLDriverPyBaseError(Exception):
    """Base PSQL-Rust-Engine exception."""

class DBPoolError(RustPSQLDriverPyBaseError):
    """Error if something goes wrong with Database Pool.

    It has verbose error message.
    """

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

class DBTransactionError(RustPSQLDriverPyBaseError):
    """Error if something goes wrong with `Transaction`.

    It has verbose error message.
    """

class DBPoolConfigurationError(RustPSQLDriverPyBaseError):
    """Error if configuration of the database pool is unacceptable."""

class UUIDValueConvertError(RustPSQLDriverPyBaseError):
    """Error if it's impossible to convert py string UUID into rust UUID."""

class CursorError(RustPSQLDriverPyBaseError):
    """Error if something goes wrong with the cursor."""
