use pyo3::{
    create_exception,
    types::{PyModule, PyModuleMethods},
    Bound, PyResult, Python,
};

// Main exception.
create_exception!(
    psqlpy.exceptions,
    RustPSQLDriverPyBaseError,
    pyo3::exceptions::PyException
);

// Rust exceptions
// `Rust` means thats these exceptions come from external rust crates,
// not from the code of the library.
create_exception!(psqlpy.exceptions, RustException, RustPSQLDriverPyBaseError);
create_exception!(psqlpy.exceptions, DriverError, RustException);
create_exception!(psqlpy.exceptions, MacAddrParseError, RustException);
create_exception!(psqlpy.exceptions, RuntimeJoinError, RustException);

// ConnectionPool exceptions
create_exception!(
    psqlpy.exceptions,
    BaseConnectionPoolError,
    RustPSQLDriverPyBaseError
);
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
create_exception!(
    psqlpy.exceptions,
    BaseConnectionError,
    RustPSQLDriverPyBaseError
);
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
create_exception!(
    psqlpy.exceptions,
    BaseTransactionError,
    RustPSQLDriverPyBaseError
);
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
create_exception!(
    psqlpy.exceptions,
    BaseCursorError,
    RustPSQLDriverPyBaseError
);
create_exception!(psqlpy.exceptions, CursorStartError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorCloseError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorFetchError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorClosedError, BaseCursorError);

// Listener Error
create_exception!(
    psqlpy.exceptions,
    BaseListenerError,
    RustPSQLDriverPyBaseError
);
create_exception!(psqlpy.exceptions, ListenerStartError, BaseListenerError);
create_exception!(psqlpy.exceptions, ListenerClosedError, BaseListenerError);
create_exception!(psqlpy.exceptions, ListenerCallbackError, BaseListenerError);

// Inner exceptions
create_exception!(
    psqlpy.exceptions,
    RustToPyValueMappingError,
    RustPSQLDriverPyBaseError
);
create_exception!(
    psqlpy.exceptions,
    PyToRustValueMappingError,
    RustPSQLDriverPyBaseError
);

create_exception!(
    psqlpy.exceptions,
    UUIDValueConvertError,
    RustPSQLDriverPyBaseError
);

create_exception!(
    psqlpy.exceptions,
    MacAddrConversionError,
    RustPSQLDriverPyBaseError
);

create_exception!(psqlpy.exceptions, SSLError, RustPSQLDriverPyBaseError);

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::too_many_lines)]
pub fn python_exceptions_module(py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add(
        "RustPSQLDriverPyBaseError",
        py.get_type::<RustPSQLDriverPyBaseError>(),
    )?;

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
