use pyo3::{types::PyAnyMethods, FromPyObject, PyAny};

use crate::exceptions::rust_errors::RustPSQLDriverError;

/// Extract a value from a Python object, raising an error if missing or invalid
///
/// # Errors
/// This function will return `Err` in the following cases:
/// - The Python object does not have the specified attribute
/// - The attribute exists but cannot be extracted into the specified Rust type
pub fn extract_value_from_python_object_or_raise<'py, T>(
    parameter: &'py pyo3::Bound<'_, PyAny>,
    attr_name: &str,
) -> Result<T, RustPSQLDriverError>
where
    T: FromPyObject<'py>,
{
    parameter
        .getattr(attr_name)
        .ok()
        .and_then(|attr| attr.extract::<T>().ok())
        .ok_or_else(|| {
            RustPSQLDriverError::PyToRustValueConversionError("Invalid attribute".into())
        })
}
