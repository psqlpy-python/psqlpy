use std::str::FromStr;

use macaddr::{MacAddr6, MacAddr8};
use pyo3::{
    pyclass, pymethods,
    types::{PyModule, PyModuleMethods},
    Bound, Py, PyAny, PyResult, Python,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{exceptions::rust_errors::RustPSQLDriverPyResult, value_converter::build_serde_value};

macro_rules! build_python_type {
    ($st_name:ident, $rust_type:ty) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner_value: $rust_type,
        }

        impl $st_name {
            #[must_use]
            pub fn retrieve_value(&self) -> $rust_type {
                self.inner_value
            }
        }

        #[pymethods]
        impl $st_name {
            #[new]
            #[must_use]
            pub fn new_class(inner_value: $rust_type) -> Self {
                Self { inner_value }
            }

            #[must_use]
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
    #[must_use]
    pub fn inner(&self) -> Uuid {
        self.inner
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyText {
    inner: String,
}

impl PyText {
    #[must_use]
    pub fn inner(&self) -> String {
        self.inner.clone()
    }
}

#[pymethods]
impl PyText {
    /// Create new PyText from Python str.
    #[new]
    #[allow(clippy::missing_errors_doc)]
    #[must_use]
    pub fn new_pytext(text_value: String) -> Self {
        Self { inner: text_value }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyVarChar {
    inner: String,
}

impl PyVarChar {
    #[must_use]
    pub fn inner(&self) -> String {
        self.inner.clone()
    }
}

#[pymethods]
impl PyVarChar {
    /// Create new PyVarChar from Python str.
    #[new]
    #[allow(clippy::missing_errors_doc)]
    #[must_use]
    pub fn new_varchar(text_value: String) -> Self {
        Self { inner: text_value }
    }
}

#[pymethods]
impl PyUUID {
    /// Create new uuid from Python str.
    ///
    /// # Errors
    /// May return Err Result if cannot convert python string
    /// into rust Uuid.
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_uuid(uuid_value: &str) -> RustPSQLDriverPyResult<Self> {
        Ok(Self {
            inner: Uuid::from_str(uuid_value)?,
        })
    }
}

macro_rules! build_json_py_type {
    ($st_name:ident, $rust_type:ty) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner: $rust_type,
        }

        impl $st_name {
            #[must_use]
            pub fn inner(&self) -> &$rust_type {
                &self.inner
            }
        }

        #[pymethods]
        impl $st_name {
            #[new]
            #[allow(clippy::missing_errors_doc)]
            pub fn new_class(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
                Ok(Self {
                    inner: build_serde_value(value)?,
                })
            }

            #[must_use]
            pub fn __str__(&self) -> String {
                format!("{}, {}", stringify!($st_name), self.inner)
            }
        }
    };
}

build_json_py_type!(PyJSONB, Value);
build_json_py_type!(PyJSON, Value);

macro_rules! build_macaddr_type {
    ($st_name:ident, $rust_type:ty) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner: $rust_type,
        }

        impl $st_name {
            #[must_use]
            pub fn inner(self) -> $rust_type {
                self.inner
            }
        }

        #[pymethods]
        impl $st_name {
            #[new]
            #[allow(clippy::missing_errors_doc)]
            pub fn new_class(value: &str) -> RustPSQLDriverPyResult<Self> {
                Ok(Self {
                    inner: <$rust_type>::from_str(value)?,
                })
            }
        }
    };
}

build_macaddr_type!(PyMacAddr6, MacAddr6);
build_macaddr_type!(PyMacAddr8, MacAddr8);

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::missing_errors_doc)]
pub fn extra_types_module(_py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add_class::<SmallInt>()?;
    pymod.add_class::<Integer>()?;
    pymod.add_class::<BigInt>()?;
    pymod.add_class::<PyUUID>()?;
    pymod.add_class::<PyText>()?;
    pymod.add_class::<PyVarChar>()?;
    pymod.add_class::<PyJSONB>()?;
    pymod.add_class::<PyJSON>()?;
    pymod.add_class::<PyMacAddr6>()?;
    pymod.add_class::<PyMacAddr8>()?;
    Ok(())
}
