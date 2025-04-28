use postgres_types::FromSql;
use serde_json::{json, Value};
use uuid::Uuid;

use pyo3::{
    types::{PyAnyMethods, PyDict, PyList},
    Bound, FromPyObject, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};
use tokio_postgres::types::Type;

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    value_converter::funcs::{from_python::py_to_rust, to_python::build_python_from_serde_value},
};

/// Struct for Value.
///
/// We use custom struct because we need to implement external traits
/// to it.
#[derive(Clone)]
pub struct InternalSerdeValue(Value);

impl<'a> FromPyObject<'a> for InternalSerdeValue {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        let serde_value = build_serde_value(ob.clone().unbind())?;

        Ok(InternalSerdeValue(serde_value))
    }
}

impl ToPyObject for InternalSerdeValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match build_python_from_serde_value(py, self.0.clone()) {
            Ok(ok_value) => ok_value,
            Err(_) => py.None(),
        }
    }
}

impl<'a> FromSql<'a> for InternalSerdeValue {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InternalSerdeValue(<Value as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

/// Convert python List of Dict type or just Dict into serde `Value`.
///
/// # Errors
/// May return error if cannot convert Python type into Rust one.
#[allow(clippy::needless_pass_by_value)]
pub fn build_serde_value(value: Py<PyAny>) -> RustPSQLDriverPyResult<Value> {
    Python::with_gil(|gil| {
        let bind_value = value.bind(gil);
        if bind_value.is_instance_of::<PyList>() {
            let mut result_vec: Vec<Value> = vec![];

            let params = bind_value.extract::<Vec<Py<PyAny>>>()?;

            for inner in params {
                let inner_bind = inner.bind(gil);
                if inner_bind.is_instance_of::<PyDict>() {
                    let python_dto = py_to_rust(inner_bind)?;
                    result_vec.push(python_dto.to_serde_value()?);
                } else if inner_bind.is_instance_of::<PyList>() {
                    let serde_value = build_serde_value(inner)?;
                    result_vec.push(serde_value);
                } else {
                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                        "PyJSON must have dicts.".to_string(),
                    ));
                }
            }
            Ok(json!(result_vec))
        } else if bind_value.is_instance_of::<PyDict>() {
            return py_to_rust(bind_value)?.to_serde_value();
        } else {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                "PyJSON must be dict value.".to_string(),
            ));
        }
    })
}
