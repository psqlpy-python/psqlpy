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

// Cursor exceptions
create_exception!(psqlpy.exceptions, BaseCursorError, BaseTransactionError);
create_exception!(psqlpy.exceptions, CursorStartError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorCloseError, BaseCursorError);
create_exception!(psqlpy.exceptions, CursorFetchError, BaseCursorError);

// Inner exceptions
create_exception!(psqlpy.exceptions, DBPoolError, RustPSQLDriverPyBaseError);
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
    TransactionError,
    RustPSQLDriverPyBaseError
);
create_exception!(
    psqlpy.exceptions,
    DBPoolConfigurationError,
    RustPSQLDriverPyBaseError
);

create_exception!(
    psqlpy.exceptions,
    UUIDValueConvertError,
    RustPSQLDriverPyBaseError
);

create_exception!(
    psqlpy.exceptions,
    MacAddr6ConversionError,
    RustPSQLDriverPyBaseError
);

create_exception!(
    psqlpy.exceptions,
    RustRuntimeJoinError,
    RustPSQLDriverPyBaseError
);

create_exception!(psqlpy.exceptions, CursorError, RustPSQLDriverPyBaseError);

#[allow(clippy::missing_errors_doc)]
pub fn python_exceptions_module(py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add(
        "RustPSQLDriverPyBaseError",
        py.get_type_bound::<RustPSQLDriverPyBaseError>(),
    )?;
    pymod.add("DBPoolError", py.get_type_bound::<DBPoolError>())?;
    pymod.add(
        "RustToPyValueMappingError",
        py.get_type_bound::<RustToPyValueMappingError>(),
    )?;
    pymod.add(
        "PyToRustValueMappingError",
        py.get_type_bound::<PyToRustValueMappingError>(),
    )?;
    pymod.add("TransactionError", py.get_type_bound::<TransactionError>())?;
    pymod.add(
        "DBPoolConfigurationError",
        py.get_type_bound::<DBPoolConfigurationError>(),
    )?;
    pymod.add(
        "UUIDValueConvertError",
        py.get_type_bound::<UUIDValueConvertError>(),
    )?;
    pymod.add("CursorError", py.get_type_bound::<CursorError>())?;
    pymod.add(
        "MacAddr6ConversionError",
        py.get_type_bound::<MacAddr6ConversionError>(),
    )?;
    pymod.add(
        "RustRuntimeJoinError",
        py.get_type_bound::<MacAddr6ConversionError>(),
    )?;
    Ok(())
}
