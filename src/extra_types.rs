use std::str::FromStr;

use geo_types::{Line as LineSegment, LineString, Point, Rect};
use macaddr::{MacAddr6, MacAddr8};
use pyo3::{
    pyclass, pymethods,
    types::{PyModule, PyModuleMethods},
    Bound, Py, PyAny, PyResult, Python,
};
use serde_json::Value;

use crate::{
    additional_types::{Circle, Line},
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    value_converter::{
        build_flat_geo_coords, build_geo_coords, build_serde_value,
        py_sequence_into_postgres_array, PythonDTO,
    },
};

#[pyclass]
#[derive(Clone)]
pub struct PgVector(Vec<f32>);

#[pymethods]
impl PgVector {
    #[new]
    fn new(vector: Vec<f32>) -> Self {
        Self(vector)
    }
}

impl PgVector {
    #[must_use]
    pub fn inner_value(self) -> Vec<f32> {
        self.0
    }
}

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
build_python_type!(Money, i64);
build_python_type!(Float32, f32);
build_python_type!(Float64, f64);

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

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCustomType {
    inner: Vec<u8>,
}

impl PyCustomType {
    #[must_use]
    pub fn inner(&self) -> Vec<u8> {
        self.inner.clone()
    }
}

#[pymethods]
impl PyCustomType {
    #[new]
    fn new_class(type_bytes: Vec<u8>) -> Self {
        PyCustomType { inner: type_bytes }
    }
}

macro_rules! build_geo_type {
    ($st_name:ident, $rust_type:ty) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner: $rust_type,
        }

        impl $st_name {
            #[must_use]
            pub fn retrieve_value(&self) -> $rust_type {
                self.inner.clone()
            }
        }
    };
}

build_geo_type!(PyPoint, Point);
build_geo_type!(PyBox, Rect);
build_geo_type!(PyPath, LineString);
build_geo_type!(PyLine, Line);
build_geo_type!(PyLineSegment, LineSegment);
build_geo_type!(PyCircle, Circle);

#[pymethods]
impl PyPoint {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_point(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
        let point_coords = build_geo_coords(value, Some(1))?;

        Ok(Self {
            inner: Point::from(point_coords[0]),
        })
    }
}

#[pymethods]
impl PyBox {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_box(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
        let box_coords = build_geo_coords(value, Some(2))?;

        Ok(Self {
            inner: Rect::new(box_coords[0], box_coords[1]),
        })
    }
}

#[pymethods]
impl PyPath {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_path(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
        let path_coords = build_geo_coords(value, None)?;

        Ok(Self {
            inner: LineString::new(path_coords),
        })
    }
}

#[pymethods]
impl PyLine {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_line(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
        let line_coords = build_flat_geo_coords(value, Some(3))?;

        Ok(Self {
            inner: Line::new(line_coords[0], line_coords[1], line_coords[2]),
        })
    }
}

#[pymethods]
impl PyLineSegment {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_line_segment(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
        let line_segment_coords = build_geo_coords(value, Some(2))?;

        Ok(Self {
            inner: LineSegment::new(line_segment_coords[0], line_segment_coords[1]),
        })
    }
}

#[pymethods]
impl PyCircle {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_circle(value: Py<PyAny>) -> RustPSQLDriverPyResult<Self> {
        let circle_coords = build_flat_geo_coords(value, Some(3))?;
        Ok(Self {
            inner: Circle::new(circle_coords[0], circle_coords[1], circle_coords[2]),
        })
    }
}

macro_rules! build_array_type {
    ($st_name:ident, $kind:path) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner: Py<PyAny>,
        }

        #[pymethods]
        impl $st_name {
            #[new]
            #[must_use]
            pub fn new_class(inner: Py<PyAny>) -> Self {
                Self { inner }
            }
        }

        impl $st_name {
            #[must_use]
            pub fn inner(&self) -> Py<PyAny> {
                self.inner.clone()
            }

            /// Convert incoming sequence from python to internal `PythonDTO`.
            ///
            /// # Errors
            /// May return Err Result if cannot convert sequence to array.
            pub fn _convert_to_python_dto(&self) -> RustPSQLDriverPyResult<PythonDTO> {
                return Python::with_gil(|gil| {
                    let binding = &self.inner;
                    let bound_inner = Ok::<&pyo3::Bound<'_, pyo3::PyAny>, RustPSQLDriverError>(
                        binding.bind(gil),
                    )?;
                    Ok::<PythonDTO, RustPSQLDriverError>($kind(py_sequence_into_postgres_array(
                        bound_inner,
                    )?))
                });
            }
        }
    };
}

build_array_type!(BoolArray, PythonDTO::PyBoolArray);
build_array_type!(UUIDArray, PythonDTO::PyUuidArray);
build_array_type!(VarCharArray, PythonDTO::PyVarCharArray);
build_array_type!(TextArray, PythonDTO::PyTextArray);
build_array_type!(Int16Array, PythonDTO::PyInt16Array);
build_array_type!(Int32Array, PythonDTO::PyInt32Array);
build_array_type!(Int64Array, PythonDTO::PyInt64Array);
build_array_type!(Float32Array, PythonDTO::PyFloat32Array);
build_array_type!(Float64Array, PythonDTO::PyFloat64Array);
build_array_type!(MoneyArray, PythonDTO::PyMoneyArray);
build_array_type!(IpAddressArray, PythonDTO::PyIpAddressArray);
build_array_type!(JSONBArray, PythonDTO::PyJSONBArray);
build_array_type!(JSONArray, PythonDTO::PyJSONArray);
build_array_type!(DateArray, PythonDTO::PyDateArray);
build_array_type!(TimeArray, PythonDTO::PyTimeArray);
build_array_type!(DateTimeArray, PythonDTO::PyDateTimeArray);
build_array_type!(DateTimeTZArray, PythonDTO::PyDateTimeTZArray);
build_array_type!(MacAddr6Array, PythonDTO::PyMacAddr6Array);
build_array_type!(MacAddr8Array, PythonDTO::PyMacAddr8Array);
build_array_type!(NumericArray, PythonDTO::PyNumericArray);
build_array_type!(PointArray, PythonDTO::PyPointArray);
build_array_type!(BoxArray, PythonDTO::PyBoxArray);
build_array_type!(PathArray, PythonDTO::PyPathArray);
build_array_type!(LineArray, PythonDTO::PyLineArray);
build_array_type!(LsegArray, PythonDTO::PyLsegArray);
build_array_type!(CircleArray, PythonDTO::PyCircleArray);
build_array_type!(IntervalArray, PythonDTO::PyIntervalArray);

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::missing_errors_doc)]
pub fn extra_types_module(_py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add_class::<SmallInt>()?;
    pymod.add_class::<Integer>()?;
    pymod.add_class::<BigInt>()?;
    pymod.add_class::<Money>()?;
    pymod.add_class::<Float32>()?;
    pymod.add_class::<Float64>()?;
    pymod.add_class::<PyText>()?;
    pymod.add_class::<PyVarChar>()?;
    pymod.add_class::<PyJSONB>()?;
    pymod.add_class::<PyJSON>()?;
    pymod.add_class::<PyMacAddr6>()?;
    pymod.add_class::<PyMacAddr8>()?;
    pymod.add_class::<PyCustomType>()?;
    pymod.add_class::<PyPoint>()?;
    pymod.add_class::<PyBox>()?;
    pymod.add_class::<PyPath>()?;
    pymod.add_class::<PyLine>()?;
    pymod.add_class::<PyLineSegment>()?;
    pymod.add_class::<PyCircle>()?;
    pymod.add_class::<BoolArray>()?;
    pymod.add_class::<UUIDArray>()?;
    pymod.add_class::<VarCharArray>()?;
    pymod.add_class::<TextArray>()?;
    pymod.add_class::<Int16Array>()?;
    pymod.add_class::<Int32Array>()?;
    pymod.add_class::<Int64Array>()?;
    pymod.add_class::<Float32Array>()?;
    pymod.add_class::<Float64Array>()?;
    pymod.add_class::<MoneyArray>()?;
    pymod.add_class::<IpAddressArray>()?;
    pymod.add_class::<JSONBArray>()?;
    pymod.add_class::<JSONArray>()?;
    pymod.add_class::<DateArray>()?;
    pymod.add_class::<TimeArray>()?;
    pymod.add_class::<DateTimeArray>()?;
    pymod.add_class::<DateTimeTZArray>()?;
    pymod.add_class::<MacAddr6Array>()?;
    pymod.add_class::<MacAddr8Array>()?;
    pymod.add_class::<NumericArray>()?;
    pymod.add_class::<PointArray>()?;
    pymod.add_class::<BoxArray>()?;
    pymod.add_class::<PathArray>()?;
    pymod.add_class::<LineArray>()?;
    pymod.add_class::<LsegArray>()?;
    pymod.add_class::<CircleArray>()?;
    pymod.add_class::<IntervalArray>()?;
    pymod.add_class::<PgVector>()?;
    Ok(())
}
