use pyo3::{create_exception, types::PyModule, PyResult, Python};

create_exception!(
    psqlpy.exceptions,
    RustPSQLDriverPyBaseError,
    pyo3::exceptions::PyException
);
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
    DBTransactionError,
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

create_exception!(psqlpy.exceptions, CursorError, RustPSQLDriverPyBaseError);

#[allow(clippy::missing_errors_doc)]
pub fn python_exceptions_module(py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add(
        "RustPSQLDriverPyBaseError",
        py.get_type::<RustPSQLDriverPyBaseError>(),
    )?;
    pymod.add("DBPoolError", py.get_type::<DBPoolError>())?;
    pymod.add(
        "RustToPyValueMappingError",
        py.get_type::<RustToPyValueMappingError>(),
    )?;
    pymod.add(
        "PyToRustValueMappingError",
        py.get_type::<PyToRustValueMappingError>(),
    )?;
    pymod.add("DBTransactionError", py.get_type::<DBTransactionError>())?;
    pymod.add(
        "DBPoolConfigurationError",
        py.get_type::<DBPoolConfigurationError>(),
    )?;
    pymod.add(
        "UUIDValueConvertError",
        py.get_type::<UUIDValueConvertError>(),
    )?;
    pymod.add("CursorError", py.get_type::<CursorError>())?;
    Ok(())
}
