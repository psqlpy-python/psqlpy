use pyo3::PyAny;

use crate::exceptions::rust_errors::PSQLPyResult;

use super::dto::enums::PythonDTO;

pub trait ToPythonDTO {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO>;
}
