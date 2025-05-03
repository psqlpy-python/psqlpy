use postgres_array::{Array, Dimension};
use postgres_types::FromSql;
use serde_json::{json, Map, Value};

use pyo3::{
    types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyTuple},
    Bound, FromPyObject, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};
use tokio_postgres::types::Type;

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    value_converter::{
        dto::enums::PythonDTO, from_python::from_python_untyped,
        to_python::build_python_from_serde_value,
    },
};

/// Struct for Value.
///
/// We use custom struct because we need to implement external traits
/// to it.
#[derive(Clone)]
pub struct InternalSerdeValue(Value);

impl<'a> FromPyObject<'a> for InternalSerdeValue {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        let serde_value = build_serde_value(ob)?;

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

fn serde_value_from_list(gil: Python<'_>, bind_value: &Bound<'_, PyAny>) -> PSQLPyResult<Value> {
    let mut result_vec: Vec<Value> = vec![];

    let params = bind_value.extract::<Vec<Py<PyAny>>>()?;

    for inner in params {
        let inner_bind = inner.bind(gil);
        if inner_bind.is_instance_of::<PyDict>() {
            let python_dto = from_python_untyped(inner_bind)?;
            result_vec.push(python_dto.to_serde_value()?);
        } else if inner_bind.is_instance_of::<PyList>() {
            let serde_value = build_serde_value(inner.bind(gil))?;
            result_vec.push(serde_value);
        } else {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                "PyJSON must have dicts.".to_string(),
            ));
        }
    }
    Ok(json!(result_vec))
}

fn serde_value_from_dict(bind_value: &Bound<'_, PyAny>) -> PSQLPyResult<Value> {
    let dict = bind_value.downcast::<PyDict>().map_err(|error| {
        RustPSQLDriverError::PyToRustValueConversionError(format!(
            "Can't cast to inner dict: {error}"
        ))
    })?;

    let mut serde_map: Map<String, Value> = Map::new();

    for dict_item in dict.items() {
        let py_list = dict_item.downcast::<PyTuple>().map_err(|error| {
            RustPSQLDriverError::PyToRustValueConversionError(format!(
                "Cannot cast to list: {error}"
            ))
        })?;

        let key = py_list.get_item(0)?.extract::<String>()?;
        let value = from_python_untyped(&py_list.get_item(1)?)?;

        serde_map.insert(key, value.to_serde_value()?);
    }

    return Ok(Value::Object(serde_map));
}

/// Convert python List of Dict type or just Dict into serde `Value`.
///
/// # Errors
/// May return error if cannot convert Python type into Rust one.
#[allow(clippy::needless_pass_by_value)]
pub fn build_serde_value(value: &Bound<'_, PyAny>) -> PSQLPyResult<Value> {
    Python::with_gil(|gil| {
        if value.is_instance_of::<PyList>() {
            return serde_value_from_list(gil, value);
        } else if value.is_instance_of::<PyDict>() {
            return serde_value_from_dict(value);
        } else {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                "PyJSON must be dict value.".to_string(),
            ));
        }
    })
}

/// Convert Array of `PythonDTO`s to serde `Value`.
///
/// It can convert multidimensional arrays.
pub fn pythondto_array_to_serde(array: Option<Array<PythonDTO>>) -> PSQLPyResult<Value> {
    match array {
        Some(array) => inner_pythondto_array_to_serde(
            array.dimensions(),
            array.iter().collect::<Vec<&PythonDTO>>().as_slice(),
            0,
            0,
        ),
        None => Ok(Value::Null),
    }
}

/// Inner conversion array of `PythonDTO`s to serde `Value`.
#[allow(clippy::cast_sign_loss)]
fn inner_pythondto_array_to_serde(
    dimensions: &[Dimension],
    data: &[&PythonDTO],
    dimension_index: usize,
    mut lower_bound: usize,
) -> PSQLPyResult<Value> {
    let current_dimension = dimensions.get(dimension_index);

    if let Some(current_dimension) = current_dimension {
        let possible_next_dimension = dimensions.get(dimension_index + 1);
        match possible_next_dimension {
            Some(next_dimension) => {
                let mut final_list: Value = Value::Array(vec![]);

                for _ in 0..current_dimension.len as usize {
                    if dimensions.get(dimension_index + 1).is_some() {
                        let inner_pylist = inner_pythondto_array_to_serde(
                            dimensions,
                            &data[lower_bound..next_dimension.len as usize + lower_bound],
                            dimension_index + 1,
                            0,
                        )?;
                        match final_list {
                            Value::Array(ref mut array) => array.push(inner_pylist),
                            _ => unreachable!(),
                        }
                        lower_bound += next_dimension.len as usize;
                    };
                }

                return Ok(final_list);
            }
            None => {
                return data.iter().map(|x| x.to_serde_value()).collect();
            }
        }
    }

    Ok(Value::Array(vec![]))
}
