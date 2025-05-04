use std::str::FromStr;

use geo_types::{Line as RustLineSegment, LineString, Point as RustPoint, Rect as RustRect};
use macaddr::{MacAddr6 as RustMacAddr6, MacAddr8 as RustMacAddr8};
use postgres_types::Type;
use pyo3::{
    pyclass, pymethods,
    types::{PyModule, PyModuleMethods},
    Bound, Py, PyAny, PyResult, Python,
};
use serde_json::Value;

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    value_converter::{
        additional_types::{Circle as RustCircle, Line as RustLine},
        dto::enums::PythonDTO,
        from_python::{build_flat_geo_coords, build_geo_coords, py_sequence_into_postgres_array},
        models::serde_value::build_serde_value,
    },
};

pub struct PythonArray;
pub struct PythonDecimal;
pub struct PythonUUID;
pub struct PythonEnum;

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
    pub fn inner(self) -> Vec<f32> {
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
            pub fn inner(&self) -> $rust_type {
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
pub struct Text {
    inner: String,
}

impl Text {
    #[must_use]
    pub fn inner(&self) -> String {
        self.inner.clone()
    }
}

#[pymethods]
impl Text {
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
pub struct VarChar {
    inner: String,
}

impl VarChar {
    #[must_use]
    pub fn inner(&self) -> String {
        self.inner.clone()
    }
}

#[pymethods]
impl VarChar {
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
            pub fn inner(&self) -> $rust_type {
                self.inner.clone()
            }

            #[must_use]
            pub fn inner_ref(&self) -> &$rust_type {
                &self.inner
            }
        }

        #[pymethods]
        impl $st_name {
            #[new]
            #[allow(clippy::missing_errors_doc)]
            pub fn new_class(value: &Bound<'_, PyAny>) -> PSQLPyResult<Self> {
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

build_json_py_type!(JSONB, Value);
build_json_py_type!(JSON, Value);

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
            pub fn new_class(value: &str) -> PSQLPyResult<Self> {
                Ok(Self {
                    inner: <$rust_type>::from_str(value)?,
                })
            }
        }
    };
}

build_macaddr_type!(MacAddr6, RustMacAddr6);
build_macaddr_type!(MacAddr8, RustMacAddr8);

#[pyclass]
#[derive(Clone, Debug)]
pub struct CustomType {
    inner: Vec<u8>,
}

impl CustomType {
    #[must_use]
    pub fn inner(&self) -> Vec<u8> {
        self.inner.clone()
    }
}

#[pymethods]
impl CustomType {
    #[new]
    fn new_class(type_bytes: Vec<u8>) -> Self {
        CustomType { inner: type_bytes }
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
            pub fn inner(&self) -> $rust_type {
                self.inner.clone()
            }
        }
    };
}

build_geo_type!(Point, RustPoint);
build_geo_type!(Box, RustRect);
build_geo_type!(Path, LineString);
build_geo_type!(Line, RustLine);
build_geo_type!(LineSegment, RustLineSegment);
build_geo_type!(Circle, RustCircle);

#[pymethods]
impl Point {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_point(value: Py<PyAny>) -> PSQLPyResult<Self> {
        let point_coords = build_geo_coords(value, Some(1))?;

        Ok(Self {
            inner: RustPoint::from(point_coords[0]),
        })
    }
}

#[pymethods]
impl Box {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_box(value: Py<PyAny>) -> PSQLPyResult<Self> {
        let box_coords = build_geo_coords(value, Some(2))?;

        Ok(Self {
            inner: RustRect::new(box_coords[0], box_coords[1]),
        })
    }
}

#[pymethods]
impl Path {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_path(value: Py<PyAny>) -> PSQLPyResult<Self> {
        let path_coords = build_geo_coords(value, None)?;

        Ok(Self {
            inner: LineString::new(path_coords),
        })
    }
}

#[pymethods]
impl Line {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_line(value: Py<PyAny>) -> PSQLPyResult<Self> {
        let line_coords = build_flat_geo_coords(value, Some(3))?;

        Ok(Self {
            inner: RustLine::new(line_coords[0], line_coords[1], line_coords[2]),
        })
    }
}

#[pymethods]
impl LineSegment {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_line_segment(value: Py<PyAny>) -> PSQLPyResult<Self> {
        let line_segment_coords = build_geo_coords(value, Some(2))?;

        Ok(Self {
            inner: RustLineSegment::new(line_segment_coords[0], line_segment_coords[1]),
        })
    }
}

#[pymethods]
impl Circle {
    #[new]
    #[allow(clippy::missing_errors_doc)]
    pub fn new_circle(value: Py<PyAny>) -> PSQLPyResult<Self> {
        let circle_coords = build_flat_geo_coords(value, Some(3))?;
        Ok(Self {
            inner: RustCircle::new(circle_coords[0], circle_coords[1], circle_coords[2]),
        })
    }
}

macro_rules! build_array_type {
    ($st_name:ident, $kind:path, $elem_kind:path) => {
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

            pub fn element_type() -> Type {
                $elem_kind
            }

            /// Convert incoming sequence from python to internal `PythonDTO`.
            ///
            /// # Errors
            /// May return Err Result if cannot convert sequence to array.
            pub fn _convert_to_python_dto(&self, elem_type: &Type) -> PSQLPyResult<PythonDTO> {
                return Python::with_gil(|gil| {
                    let binding = &self.inner;
                    let bound_inner = Ok::<&pyo3::Bound<'_, pyo3::PyAny>, RustPSQLDriverError>(
                        binding.bind(gil),
                    )?;
                    Ok::<PythonDTO, RustPSQLDriverError>($kind(py_sequence_into_postgres_array(
                        bound_inner,
                        elem_type,
                    )?))
                });
            }
        }
    };
}

build_array_type!(BoolArray, PythonDTO::PyBoolArray, Type::BOOL);
build_array_type!(UUIDArray, PythonDTO::PyUuidArray, Type::UUID);
build_array_type!(VarCharArray, PythonDTO::PyVarCharArray, Type::VARCHAR);
build_array_type!(TextArray, PythonDTO::PyTextArray, Type::TEXT);
build_array_type!(Int16Array, PythonDTO::PyInt16Array, Type::INT2);
build_array_type!(Int32Array, PythonDTO::PyInt32Array, Type::INT4);
build_array_type!(Int64Array, PythonDTO::PyInt64Array, Type::INT8);
build_array_type!(Float32Array, PythonDTO::PyFloat32Array, Type::FLOAT4);
build_array_type!(Float64Array, PythonDTO::PyFloat64Array, Type::FLOAT8);
build_array_type!(MoneyArray, PythonDTO::PyMoneyArray, Type::MONEY);
build_array_type!(IpAddressArray, PythonDTO::PyIpAddressArray, Type::INET);
build_array_type!(JSONBArray, PythonDTO::PyJSONBArray, Type::JSONB);
build_array_type!(JSONArray, PythonDTO::PyJSONArray, Type::JSON);
build_array_type!(DateArray, PythonDTO::PyDateArray, Type::DATE);
build_array_type!(TimeArray, PythonDTO::PyTimeArray, Type::TIME);
build_array_type!(DateTimeArray, PythonDTO::PyDateTimeArray, Type::TIMESTAMP);
build_array_type!(
    DateTimeTZArray,
    PythonDTO::PyDateTimeTZArray,
    Type::TIMESTAMPTZ
);
build_array_type!(MacAddr6Array, PythonDTO::PyMacAddr6Array, Type::MACADDR);
build_array_type!(MacAddr8Array, PythonDTO::PyMacAddr8Array, Type::MACADDR8);
build_array_type!(NumericArray, PythonDTO::PyNumericArray, Type::NUMERIC);
build_array_type!(PointArray, PythonDTO::PyPointArray, Type::POINT);
build_array_type!(BoxArray, PythonDTO::PyBoxArray, Type::BOX);
build_array_type!(PathArray, PythonDTO::PyPathArray, Type::PATH);
build_array_type!(LineArray, PythonDTO::PyLineArray, Type::LINE);
build_array_type!(LsegArray, PythonDTO::PyLsegArray, Type::LSEG);
build_array_type!(CircleArray, PythonDTO::PyCircleArray, Type::CIRCLE);
build_array_type!(IntervalArray, PythonDTO::PyIntervalArray, Type::INTERVAL);

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::missing_errors_doc)]
pub fn extra_types_module(_py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add_class::<SmallInt>()?;
    pymod.add_class::<Integer>()?;
    pymod.add_class::<BigInt>()?;
    pymod.add_class::<Money>()?;
    pymod.add_class::<Float32>()?;
    pymod.add_class::<Float64>()?;
    pymod.add_class::<Text>()?;
    pymod.add_class::<VarChar>()?;
    pymod.add_class::<JSONB>()?;
    pymod.add_class::<JSON>()?;
    pymod.add_class::<MacAddr6>()?;
    pymod.add_class::<MacAddr8>()?;
    pymod.add_class::<CustomType>()?;
    pymod.add_class::<Point>()?;
    pymod.add_class::<Box>()?;
    pymod.add_class::<Path>()?;
    pymod.add_class::<Line>()?;
    pymod.add_class::<LineSegment>()?;
    pymod.add_class::<Circle>()?;
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
