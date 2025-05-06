use pyo3::{
    create_exception,
    types::{PyModule, PyModuleMethods},
    Bound, PyResult, Python,
};

// Exception raised for important warnings like data truncations while inserting, etc.
create_exception!(
    psqlpy.exceptions,
    WarningError,
    pyo3::exceptions::PyException
);

// Exception that is the base class of all other error exceptions.
// You can use this to catch all errors with one single except statement.
create_exception!(psqlpy.exceptions, Error, pyo3::exceptions::PyException);

// Exception raised for errors that are related to the
// database interface rather than the database itself.
create_exception!(psqlpy.exceptions, InterfaceError, Error);

// Exception raised for errors that are related to the database.
create_exception!(psqlpy.exceptions, DatabaseError, Error);

// Exception raised for errors that are due to problems with
// the processed data like division by zero, numeric value out of range, etc.
create_exception!(psqlpy.exceptions, DataError, DatabaseError);

// Exception raised for errors that are related to the database’s operation
// and not necessarily under the control of the programmer,
// e.g. an unexpected disconnect occurs, the data source name is not found,
// a transaction could not be processed, a memory allocation error
// occurred during processing, etc.
create_exception!(psqlpy.exceptions, OperationalError, DatabaseError);

// Exception raised when the relational integrity of the
// database is affected, e.g. a foreign key check fails.
create_exception!(psqlpy.exceptions, IntegrityError, DatabaseError);

// Exception raised when the database encounters an internal error,
// e.g. the cursor is not valid anymore, the transaction is out of sync, etc.
create_exception!(psqlpy.exceptions, InternalError, DatabaseError);

// Exception raised for programming errors, e.g. table not found or
// already exists, syntax error in the SQL statement,
// wrong number of parameters specified, etc.
create_exception!(psqlpy.exceptions, ProgrammingError, DatabaseError);
// Exception raised in case a method or database API was used which
// is not supported by the database, e.g. requesting a .rollback()
// on a connection that does not support transaction
// or has transactions turned off.
create_exception!(psqlpy.exceptions, NotSupportedError, DatabaseError);

// Rust exceptions
// `Rust` means thats these exceptions come from external rust crates,
// not from the code of the library.
create_exception!(psqlpy.exceptions, MacAddrParseError, DataError);
create_exception!(psqlpy.exceptions, RuntimeJoinError, DataError);

// ConnectionPool exceptions
create_exception!(psqlpy.exceptions, BaseConnectionPoolError, InterfaceError);
create_exception!(
    psqlpy.exceptions,
    ConnectionPoolBuildError,
    BaseConnectionPoolError
);
create_exception!(
    psqlpy.exceptions,
    ConnectionPoolConfigurationError,
    BaseConnectionPoolError
);
create_exception!(
    psqlpy.exceptions,
    ConnectionPoolExecuteError,
    BaseConnectionPoolError
);

// Connection exceptions
create_exception!(psqlpy.exceptions, BaseConnectionError, InterfaceError);
create_exception!(
    psqlpy.exceptions,
    ConnectionExecuteError,
    BaseConnectionError
);
create_exception!(
    psqlpy.exceptions,
    ConnectionClosedError,
    BaseConnectionError
);

// Transaction exceptions
create_exception!(psqlpy.exceptions, BaseTransactionError, InterfaceError);
create_exception!(
    psqlpy.exceptions,
    TransactionBeginError,
    BaseTransactionError
);
create_exception!(
    psqlpy.exceptions,
    TransactionCommitError,
    BaseTransactionError
);
create_exception!(
    psqlpy.exceptions,
    TransactionRollbackError,
    BaseTransactionError
);
create_exception!(
    psqlpy.exceptions,
    TransactionSavepointError,
    BaseTransactionError
);
create_exception!(
    psqlpy.exceptions,
    TransactionExecuteError,
    BaseTransactionError
);
create_exception!(
    psqlpy.exceptions,
    TransactionClosedError,
    BaseTransactionError
);

// Cursor exceptions
create_exception!(psqlpy.exceptions, BaseCursorError, InterfaceError);
create_exception!(psqlpy.exceptions, CursorStartError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorCloseError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorFetchError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorClosedError, BaseCursorError);

// Listener Error
create_exception!(psqlpy.exceptions, BaseListenerError, InterfaceError);
create_exception!(psqlpy.exceptions, ListenerStartError, BaseListenerError);
create_exception!(psqlpy.exceptions, ListenerClosedError, BaseListenerError);
create_exception!(psqlpy.exceptions, ListenerCallbackError, BaseListenerError);

// Inner exceptions
create_exception!(psqlpy.exceptions, RustToPyValueMappingError, DataError);
create_exception!(psqlpy.exceptions, PyToRustValueMappingError, DataError);

create_exception!(psqlpy.exceptions, UUIDValueConvertError, DataError);

create_exception!(psqlpy.exceptions, MacAddrConversionError, DataError);

create_exception!(psqlpy.exceptions, SSLError, DatabaseError);

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::too_many_lines)]
pub fn python_exceptions_module(py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add("WarningError", py.get_type::<WarningError>())?;
    pymod.add("Error", py.get_type::<Error>())?;
    pymod.add("InterfaceError", py.get_type::<InterfaceError>())?;
    pymod.add("DatabaseError", py.get_type::<DatabaseError>())?;
    pymod.add("DataError", py.get_type::<DataError>())?;
    pymod.add("OperationalError", py.get_type::<OperationalError>())?;
    pymod.add("IntegrityError", py.get_type::<IntegrityError>())?;
    pymod.add("InternalError", py.get_type::<InternalError>())?;
    pymod.add("ProgrammingError", py.get_type::<ProgrammingError>())?;
    pymod.add("NotSupportedError", py.get_type::<NotSupportedError>())?;

    pymod.add(
        "BaseConnectionPoolError",
        py.get_type::<BaseConnectionPoolError>(),
    )?;
    pymod.add(
        "ConnectionPoolBuildError",
        py.get_type::<ConnectionPoolBuildError>(),
    )?;
    pymod.add(
        "ConnectionPoolConfigurationError",
        py.get_type::<ConnectionPoolConfigurationError>(),
    )?;
    pymod.add(
        "ConnectionPoolExecuteError",
        py.get_type::<ConnectionPoolExecuteError>(),
    )?;

    pymod.add("BaseConnectionError", py.get_type::<BaseConnectionError>())?;
    pymod.add(
        "ConnectionExecuteError",
        py.get_type::<ConnectionExecuteError>(),
    )?;
    pymod.add(
        "ConnectionClosedError",
        py.get_type::<ConnectionClosedError>(),
    )?;

    pymod.add(
        "BaseTransactionError",
        py.get_type::<BaseTransactionError>(),
    )?;
    pymod.add(
        "TransactionBeginError",
        py.get_type::<TransactionBeginError>(),
    )?;
    pymod.add(
        "TransactionCommitError",
        py.get_type::<TransactionCommitError>(),
    )?;
    pymod.add(
        "TransactionRollbackError",
        py.get_type::<TransactionRollbackError>(),
    )?;
    pymod.add(
        "TransactionSavepointError",
        py.get_type::<TransactionSavepointError>(),
    )?;
    pymod.add(
        "TransactionExecuteError",
        py.get_type::<TransactionExecuteError>(),
    )?;
    pymod.add(
        "TransactionClosedError",
        py.get_type::<TransactionClosedError>(),
    )?;

    pymod.add("BaseCursorError", py.get_type::<BaseCursorError>())?;
    pymod.add("CursorStartError", py.get_type::<CursorStartError>())?;
    pymod.add("CursorCloseError", py.get_type::<CursorCloseError>())?;
    pymod.add("CursorFetchError", py.get_type::<CursorFetchError>())?;
    pymod.add("CursorClosedError", py.get_type::<CursorClosedError>())?;

    pymod.add(
        "RustToPyValueMappingError",
        py.get_type::<RustToPyValueMappingError>(),
    )?;
    pymod.add(
        "PyToRustValueMappingError",
        py.get_type::<PyToRustValueMappingError>(),
    )?;
    pymod.add(
        "UUIDValueConvertError",
        py.get_type::<UUIDValueConvertError>(),
    )?;
    pymod.add(
        "MacAddrConversionError",
        py.get_type::<MacAddrConversionError>(),
    )?;
    pymod.add("BaseListenerError", py.get_type::<BaseListenerError>())?;
    pymod.add("ListenerStartError", py.get_type::<ListenerStartError>())?;
    pymod.add("ListenerClosedError", py.get_type::<ListenerClosedError>())?;
    pymod.add(
        "ListenerCallbackError",
        py.get_type::<ListenerCallbackError>(),
    )?;
    Ok(())
}
