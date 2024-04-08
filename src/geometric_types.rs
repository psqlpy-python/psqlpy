use geo_types::{Point, Rect, LineString};
use pyo3::{pyclass, pymethods, types::PyModule, PyAny, PyResult, Python};

// use crate::{exceptions::rust_errors::RustPSQLDriverPyResult, value_converter::build_serde_value};

#[pyclass]
#[derive(Clone)]
pub struct PyPoint {
    inner: Point,
}

impl PyPoint {
    #[must_use]
    pub fn inner(&self) -> &Point {
        &self.inner
    }
}

// #[pymethods]
// impl PyPoint {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_point(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

#[pyclass]
#[derive(Clone)]
pub struct PyBox {
    inner: Rect,
}

impl PyBox {
    #[must_use]
    pub fn inner(&self) -> &Rect {
        &self.inner
    }
}

// #[pymethods]
// impl PyBox {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_point(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

#[pyclass]
#[derive(Clone)]
pub struct PyPath {
    inner: LineString,
}

impl PyPath {
    #[must_use]
    pub fn inner(&self) -> &LineString {
        &self.inner
    }
}

// #[pymethods]
// impl PyPath {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_point(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::missing_errors_doc)]
pub fn geometric_types_module(_py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<PyPoint>()?;
    pymod.add_class::<PyBox>()?;
    pymod.add_class::<PyPath>()?;
    Ok(())
}
