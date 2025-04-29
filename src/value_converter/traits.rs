use pyo3::PyAny;

use crate::exceptions::rust_errors::RustPSQLDriverPyResult;

use super::dto::enums::PythonDTO;

pub trait PythonToDTO {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> RustPSQLDriverPyResult<PythonDTO>;
}
