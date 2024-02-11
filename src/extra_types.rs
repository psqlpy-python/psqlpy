use std::str::FromStr;

use pyo3::{
    pyclass, pymethods,
    types::{PyDict, PyList, PyModule},
    PyAny, PyResult, Python,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    value_converter::py_to_rust,
};

macro_rules! build_python_type {
    ($st_name:ident, $rust_type:ty) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner_value: $rust_type,
        }

        impl $st_name {
            pub fn retrieve_value(&self) -> $rust_type {
                self.inner_value
            }
        }

        #[pymethods]
        impl $st_name {
            #[new]
            pub fn new_class(inner_value: $rust_type) -> Self {
                Self { inner_value }
            }

            pub fn __str__(&self) -> String {
                format!("{}, {}", stringify!($st_name), self.inner_value)
            }
        }
    };
}

build_python_type!(SmallInt, i16);
build_python_type!(Integer, i32);
build_python_type!(BigInt, i64);

#[pyclass]
#[derive(Clone)]
pub struct PyUUID {
    inner: Uuid,
}

impl PyUUID {
    pub fn inner(&self) -> Uuid {
        self.inner
    }
}

#[pymethods]
impl PyUUID {
    #[new]
    pub fn new_uuid(uuid_value: String) -> RustPSQLDriverPyResult<Self> {
        Ok(Self {
            inner: Uuid::from_str(&uuid_value)?,
        })
    }
}

pub fn build_serde_value<'a>(value: &'a PyAny) -> RustPSQLDriverPyResult<Value> {
    if value.is_instance_of::<PyList>() {
        let mut result_vec: Vec<Value> = vec![];

        let params: Vec<&PyAny> = value.extract::<Vec<&PyAny>>()?;

        for inner in params.iter() {
            if inner.is_instance_of::<PyDict>() {
                let python_dto = py_to_rust(inner)?;
                result_vec.push(python_dto.to_serde_value()?);
            } else if inner.is_instance_of::<PyList>() {
                let serde_value = build_serde_value(inner)?;
                result_vec.push(serde_value);
            } else {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
                    "PyJSON/PyJSONB supports only list of lists or list of dicts."
                )));
            }
        }
        return Ok(json!(result_vec));
    } else if value.is_instance_of::<PyDict>() {
        return py_to_rust(value)?.to_serde_value();
    } else {
        return Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
            "PyJSON must be list value."
        )));
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyJSON {
    inner: Value,
}

impl PyJSON {
    pub fn inner(&self) -> &Value {
        &self.inner
    }
}

#[pymethods]
impl PyJSON {
    #[new]
    pub fn new_uuid<'a>(value: &'a PyAny) -> RustPSQLDriverPyResult<Self> {
        return Ok(Self {
            inner: build_serde_value(value)?,
        });
    }
}

pub fn extra_types_module(_py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<SmallInt>()?;
    pymod.add_class::<Integer>()?;
    pymod.add_class::<BigInt>()?;
    pymod.add_class::<PyUUID>()?;
    pymod.add_class::<PyJSON>()?;
    Ok(())
}
