use geo_types::{Point, Rect, Line, LineString, Polygon};
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
//     pub fn new_box(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
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
//     pub fn new_path(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

#[pyclass]
#[derive(Clone)]
pub struct PyLine {
    inner: Line,
}

impl PyLine {
    #[must_use]
    pub fn inner(&self) -> &Line {
        &self.inner
    }
}

// #[pymethods]
// impl PyLine {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_path(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

#[pyclass]
#[derive(Clone)]
pub struct PyLineSegment {
    inner: Line,
}

impl PyLineSegment {
    #[must_use]
    pub fn inner(&self) -> &Line {
        &self.inner
    }
}

// #[pymethods]
// impl PyLineSegment {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_path(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

#[pyclass]
#[derive(Clone)]
pub struct PyPolygon {
    inner: Polygon,
}

impl PyPolygon {
    #[must_use]
    pub fn inner(&self) -> &Polygon {
        &self.inner
    }
}

// #[pymethods]
// impl PyPolygon {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_path(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
//         Ok(Self {
//             inner: build_serde_value(value)?,
//         })
//     }
// }

// #[pyclass]
// #[derive(Clone)]
// pub struct PyCircle {
//     inner: Polygon,
// }

// impl PyCircle {
//     #[must_use]
//     pub fn inner(&self) -> &Polygon {
//         &self.inner
//     }
// }

// #[pymethods]
// impl PyCircle {
//     #[new]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new_path(value: &PyAny) -> RustPSQLDriverPyResult<Self> {
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
    pymod.add_class::<PyLine>()?;
    pymod.add_class::<PyLineSegment>()?;
    pymod.add_class::<PyPolygon>()?;
    // pymod.add_class::<PyCircle>()?;
    Ok(())
}
