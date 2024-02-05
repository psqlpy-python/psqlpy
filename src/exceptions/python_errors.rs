use pyo3::create_exception;

create_exception!(
    rustengine.exceptions,
    RustEnginePyBaseError,
    pyo3::exceptions::PyException
);
