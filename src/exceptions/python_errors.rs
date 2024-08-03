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
pub fn python_exceptions_module(py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add(
        "RustPSQLDriverPyBaseError",
        py.get_type_bound::<RustPSQLDriverPyBaseError>(),
    )?;

    pymod.add(
        "BaseConnectionPoolError",
        py.get_type_bound::<BaseConnectionPoolError>(),
    )?;
    pymod.add(
        "ConnectionPoolBuildError",
        py.get_type_bound::<ConnectionPoolBuildError>(),
    )?;
    pymod.add(
        "ConnectionPoolConfigurationError",
        py.get_type_bound::<ConnectionPoolConfigurationError>(),
    )?;
    pymod.add(
        "ConnectionPoolExecuteError",
        py.get_type_bound::<ConnectionPoolExecuteError>(),
    )?;

    pymod.add(
        "BaseConnectionError",
        py.get_type_bound::<BaseConnectionError>(),
    )?;
    pymod.add(
        "ConnectionExecuteError",
        py.get_type_bound::<ConnectionExecuteError>(),
    )?;
    pymod.add(
        "ConnectionClosedError",
        py.get_type_bound::<ConnectionClosedError>(),
    )?;

    pymod.add(
        "BaseTransactionError",
        py.get_type_bound::<BaseTransactionError>(),
    )?;
    pymod.add(
        "TransactionBeginError",
        py.get_type_bound::<TransactionBeginError>(),
    )?;
    pymod.add(
        "TransactionCommitError",
        py.get_type_bound::<TransactionCommitError>(),
    )?;
    pymod.add(
        "TransactionRollbackError",
        py.get_type_bound::<TransactionRollbackError>(),
    )?;
    pymod.add(
        "TransactionSavepointError",
        py.get_type_bound::<TransactionSavepointError>(),
    )?;
    pymod.add(
        "TransactionExecuteError",
        py.get_type_bound::<TransactionExecuteError>(),
    )?;
    pymod.add(
        "TransactionClosedError",
        py.get_type_bound::<TransactionClosedError>(),
    )?;

    pymod.add("BaseCursorError", py.get_type_bound::<BaseCursorError>())?;
    pymod.add("CursorStartError", py.get_type_bound::<CursorStartError>())?;
    pymod.add("CursorCloseError", py.get_type_bound::<CursorCloseError>())?;
    pymod.add("CursorFetchError", py.get_type_bound::<CursorFetchError>())?;
    pymod.add(
        "CursorClosedError",
        py.get_type_bound::<CursorClosedError>(),
    )?;

    pymod.add(
        "RustToPyValueMappingError",
        py.get_type_bound::<RustToPyValueMappingError>(),
    )?;
    pymod.add(
        "PyToRustValueMappingError",
        py.get_type_bound::<PyToRustValueMappingError>(),
    )?;
    pymod.add(
        "UUIDValueConvertError",
        py.get_type_bound::<UUIDValueConvertError>(),
    )?;
    pymod.add(
        "MacAddrConversionError",
        py.get_type_bound::<MacAddrConversionError>(),
    )?;
    Ok(())
}
