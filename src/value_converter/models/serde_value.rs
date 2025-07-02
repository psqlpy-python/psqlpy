use postgres_array::{Array, Dimension};
use postgres_types::FromSql;
use serde_json::{json, Map, Value};

use pyo3::{
    types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods},
    Bound, FromPyObject, IntoPyObject, PyAny, PyResult, Python,
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

impl<'py> IntoPyObject<'py> for InternalSerdeValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = RustPSQLDriverError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match build_python_from_serde_value(py, self.0) {
            Ok(ok_value) => Ok(ok_value.bind(py).clone()),
            Err(err) => Err(err),
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

fn serde_value_from_list(_gil: Python<'_>, bind_value: &Bound<'_, PyAny>) -> PSQLPyResult<Value> {
    let py_list = bind_value.downcast::<PyList>().map_err(|e| {
        RustPSQLDriverError::PyToRustValueConversionError(format!(
            "Parameter must be a list, but it's not: {}",
            e
        ))
    })?;

    let mut result_vec: Vec<Value> = Vec::with_capacity(py_list.len());

    for item in py_list.iter() {
        if item.is_instance_of::<PyDict>() {
            let python_dto = from_python_untyped(&item)?;
            result_vec.push(python_dto.to_serde_value()?);
        } else if item.is_instance_of::<PyList>() {
            let serde_value = build_serde_value(&item)?;
            result_vec.push(serde_value);
        } else {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                "Items in JSON array must be dicts or lists.".to_string(),
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

    let dict_len = dict.len();
    let mut serde_map: Map<String, Value> = Map::with_capacity(dict_len);

    for (key, value) in dict.iter() {
        let key_str = key.extract::<String>().map_err(|error| {
            RustPSQLDriverError::PyToRustValueConversionError(format!(
                "Cannot extract dict key as string: {error}"
            ))
        })?;

        let value_dto = from_python_untyped(&value)?;
        serde_map.insert(key_str, value_dto.to_serde_value()?);
    }

    Ok(Value::Object(serde_map))
}

/// Convert python List of Dict type or just Dict into serde `Value`.
///
/// # Errors
/// May return error if cannot convert Python type into Rust one.
#[allow(clippy::needless_pass_by_value)]
pub fn build_serde_value(value: &Bound<'_, PyAny>) -> PSQLPyResult<Value> {
    Python::with_gil(|gil| {
        if value.is_instance_of::<PyList>() {
            serde_value_from_list(gil, value)
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
///
/// # Errors
/// May return error if cannot create serde value.
pub fn pythondto_array_to_serde(array: Option<Array<PythonDTO>>) -> PSQLPyResult<Value> {
    match array {
        Some(array) => {
            let data: Vec<PythonDTO> = array.iter().cloned().collect();
            inner_pythondto_array_to_serde(array.dimensions(), &data, 0, 0)
        }
        None => Ok(Value::Null),
    }
}

/// Inner conversion array of `PythonDTO`s to serde `Value`.
#[allow(clippy::cast_sign_loss)]
fn inner_pythondto_array_to_serde(
    dimensions: &[Dimension],
    data: &[PythonDTO],
    dimension_index: usize,
    data_offset: usize,
) -> PSQLPyResult<Value> {
    if dimension_index >= dimensions.len() || data_offset >= data.len() {
        return Ok(Value::Array(vec![]));
    }

    let current_dimension = &dimensions[dimension_index];
    let current_len = current_dimension.len as usize;

    if dimension_index + 1 >= dimensions.len() {
        let end_offset = (data_offset + current_len).min(data.len());
        let slice = &data[data_offset..end_offset];

        let mut result_values = Vec::with_capacity(slice.len());
        for item in slice {
            result_values.push(item.to_serde_value()?);
        }

        return Ok(Value::Array(result_values));
    }

    let mut final_array = Vec::with_capacity(current_len);

    let sub_array_size = dimensions[dimension_index + 1..]
        .iter()
        .map(|d| d.len as usize)
        .product::<usize>();

    let mut current_offset = data_offset;

    for _ in 0..current_len {
        if current_offset >= data.len() {
            break;
        }

        let inner_value =
            inner_pythondto_array_to_serde(dimensions, data, dimension_index + 1, current_offset)?;

        final_array.push(inner_value);
        current_offset += sub_array_size;
    }

    Ok(Value::Array(final_array))
}
