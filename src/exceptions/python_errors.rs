use pyo3::create_exception;

create_exception!(
    rustengine.exceptions,
    RustPSQLDriverPyBaseError,
    pyo3::exceptions::PyException
);
