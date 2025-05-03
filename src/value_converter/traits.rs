use postgres_types::Type;
use pyo3::PyAny;

use crate::exceptions::rust_errors::PSQLPyResult;

use super::dto::enums::PythonDTO;

pub trait ToPythonDTO {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO>;
}

pub trait ToPythonDTOArray {
    fn to_python_dto(
        python_param: &pyo3::Bound<'_, PyAny>,
        array_type_: Type,
    ) -> PSQLPyResult<PythonDTO>;
}
