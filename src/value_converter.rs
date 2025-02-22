use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;
use geo_types::{coord, Coord, Line as LineSegment, LineString, Point, Rect};
use itertools::Itertools;
use macaddr::{MacAddr6, MacAddr8};
use once_cell::sync::Lazy;
use pg_interval::Interval;
use postgres_types::{Field, FromSql, Kind, ToSql};
use rust_decimal::Decimal;
use serde_json::{json, Map, Value};
use std::{collections::HashMap, fmt::Debug, net::IpAddr, sync::RwLock};
use uuid::Uuid;

use bytes::{BufMut, BytesMut};
use postgres_protocol::types;
use pyo3::{
    sync::GILOnceCell,
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyDictMethods, PyFloat,
        PyInt, PyList, PyListMethods, PyMapping, PySequence, PySet, PyString, PyTime, PyTuple,
        PyType, PyTypeMethods,
    },
    Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};
use tokio_postgres::{
    types::{to_sql_checked, Type},
    Column, Row,
};

use crate::{
    additional_types::{
        Circle, Line, RustLineSegment, RustLineString, RustMacAddr6, RustMacAddr8, RustPoint,
        RustRect,
    },
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    extra_types,
};
use pgvector::Vector as PgVector;
use postgres_array::{array::Array, Dimension};

static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
static TIMEDELTA_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
static KWARGS_QUERYSTRINGS: Lazy<RwLock<HashMap<String, (String, Vec<String>)>>> =
    Lazy::new(|| RwLock::new(Default::default()));

pub type QueryParameter = (dyn ToSql + Sync);

fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    DECIMAL_CLS
        .get_or_try_init(py, || {
            let type_object = py.import("decimal")?.getattr("Decimal")?.downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
}

fn get_timedelta_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    TIMEDELTA_CLS
        .get_or_try_init(py, || {
            let type_object = py
                .import("datetime")?
                .getattr("timedelta")?
                .downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
}

/// Struct for Uuid.
///
/// We use custom struct because we need to implement external traits
/// to it.
#[derive(Clone, Copy)]
pub struct InternalUuid(Uuid);

impl<'a> FromPyObject<'a> for InternalUuid {
    fn extract_bound(obj: &Bound<'a, PyAny>) -> PyResult<Self> {
        let uuid_value = Uuid::parse_str(obj.str()?.extract::<&str>()?).map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError(
                "Cannot convert UUID Array to inner rust type, check you parameters.".into(),
            )
        })?;
        Ok(InternalUuid(uuid_value))
    }
}

impl ToPyObject for InternalUuid {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.to_string().as_str().to_object(py)
    }
}

impl<'a> FromSql<'a> for InternalUuid {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InternalUuid(<Uuid as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

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

/// Struct for Decimal.
///
/// It's necessary because we use custom forks and there is
/// no implementation of `ToPyObject` for Decimal.
struct InnerDecimal(Decimal);

impl ToPyObject for InnerDecimal {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dec_cls = get_decimal_cls(py).expect("failed to load decimal.Decimal");
        let ret = dec_cls
            .call1((self.0.to_string(),))
            .expect("failed to call decimal.Decimal(value)");
        ret.to_object(py)
    }
}

impl<'a> FromSql<'a> for InnerDecimal {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InnerDecimal(<Decimal as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

struct InnerInterval(Interval);

impl ToPyObject for InnerInterval {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let td_cls = get_timedelta_cls(py).expect("failed to load datetime.timedelta");
        let pydict = PyDict::new_bound(py);
        let months = self.0.months * 30;
        let _ = pydict.set_item("days", self.0.days + months);
        let _ = pydict.set_item("microseconds", self.0.microseconds);
        let ret = td_cls
            .call((), Some(&pydict))
            .expect("failed to call datetime.timedelta(days=<>, microseconds=<>)");
        ret.to_object(py)
    }
}

impl<'a> FromSql<'a> for InnerInterval {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InnerInterval(<Interval as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

/// Additional type for types come from Python.
///
/// It's necessary because we need to pass this
/// enum into `to_sql` method of `ToSql` trait from
/// `postgres` crate.
#[derive(Debug, Clone, PartialEq)]
pub enum PythonDTO {
    // Primitive
    PyNone,
    PyBytes(Vec<u8>),
    PyBool(bool),
    PyUUID(Uuid),
    PyVarChar(String),
    PyText(String),
    PyString(String),
    PyIntI16(i16),
    PyIntI32(i32),
    PyIntI64(i64),
    PyIntU32(u32),
    PyIntU64(u64),
    PyFloat32(f32),
    PyFloat64(f64),
    PyMoney(i64),
    PyDate(NaiveDate),
    PyTime(NaiveTime),
    PyDateTime(NaiveDateTime),
    PyDateTimeTz(DateTime<FixedOffset>),
    PyInterval(Interval),
    PyIpAddress(IpAddr),
    PyList(Vec<PythonDTO>),
    PyArray(Array<PythonDTO>),
    PyTuple(Vec<PythonDTO>),
    PyJsonb(Value),
    PyJson(Value),
    PyMacAddr6(MacAddr6),
    PyMacAddr8(MacAddr8),
    PyDecimal(Decimal),
    PyCustomType(Vec<u8>),
    PyPoint(Point),
    PyBox(Rect),
    PyPath(LineString),
    PyLine(Line),
    PyLineSegment(LineSegment),
    PyCircle(Circle),
    // Arrays
    PyBoolArray(Array<PythonDTO>),
    PyUuidArray(Array<PythonDTO>),
    PyVarCharArray(Array<PythonDTO>),
    PyTextArray(Array<PythonDTO>),
    PyInt16Array(Array<PythonDTO>),
    PyInt32Array(Array<PythonDTO>),
    PyInt64Array(Array<PythonDTO>),
    PyFloat32Array(Array<PythonDTO>),
    PyFloat64Array(Array<PythonDTO>),
    PyMoneyArray(Array<PythonDTO>),
    PyIpAddressArray(Array<PythonDTO>),
    PyJSONBArray(Array<PythonDTO>),
    PyJSONArray(Array<PythonDTO>),
    PyDateArray(Array<PythonDTO>),
    PyTimeArray(Array<PythonDTO>),
    PyDateTimeArray(Array<PythonDTO>),
    PyDateTimeTZArray(Array<PythonDTO>),
    PyMacAddr6Array(Array<PythonDTO>),
    PyMacAddr8Array(Array<PythonDTO>),
    PyNumericArray(Array<PythonDTO>),
    PyPointArray(Array<PythonDTO>),
    PyBoxArray(Array<PythonDTO>),
    PyPathArray(Array<PythonDTO>),
    PyLineArray(Array<PythonDTO>),
    PyLsegArray(Array<PythonDTO>),
    PyCircleArray(Array<PythonDTO>),
    PyIntervalArray(Array<PythonDTO>),
    // PgVector
    PyPgVector(Vec<f32>),
}

impl ToPyObject for PythonDTO {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            PythonDTO::PyNone => py.None(),
            PythonDTO::PyBool(pybool) => pybool.to_object(py),
            PythonDTO::PyString(py_string)
            | PythonDTO::PyText(py_string)
            | PythonDTO::PyVarChar(py_string) => py_string.to_object(py),
            PythonDTO::PyIntI32(pyint) => pyint.to_object(py),
            PythonDTO::PyIntI64(pyint) => pyint.to_object(py),
            PythonDTO::PyIntU64(pyint) => pyint.to_object(py),
            PythonDTO::PyFloat32(pyfloat) => pyfloat.to_object(py),
            PythonDTO::PyFloat64(pyfloat) => pyfloat.to_object(py),
            _ => unreachable!(),
        }
    }
}

impl PythonDTO {
    /// Return type of the Array for `PostgreSQL`.
    ///
    /// Since every Array must have concrete type,
    /// we must say exactly what type of array we try to pass into
    /// postgres.
    ///
    /// # Errors
    /// May return Err Result if there is no support for passed python type.
    pub fn array_type(&self) -> RustPSQLDriverPyResult<tokio_postgres::types::Type> {
        match self {
            PythonDTO::PyBool(_) => Ok(tokio_postgres::types::Type::BOOL_ARRAY),
            PythonDTO::PyUUID(_) => Ok(tokio_postgres::types::Type::UUID_ARRAY),
            PythonDTO::PyVarChar(_) | PythonDTO::PyString(_) => {
                Ok(tokio_postgres::types::Type::VARCHAR_ARRAY)
            }
            PythonDTO::PyText(_) => Ok(tokio_postgres::types::Type::TEXT_ARRAY),
            PythonDTO::PyIntI16(_) => Ok(tokio_postgres::types::Type::INT2_ARRAY),
            PythonDTO::PyIntI32(_) | PythonDTO::PyIntU32(_) => {
                Ok(tokio_postgres::types::Type::INT4_ARRAY)
            }
            PythonDTO::PyIntI64(_) => Ok(tokio_postgres::types::Type::INT8_ARRAY),
            PythonDTO::PyFloat32(_) => Ok(tokio_postgres::types::Type::FLOAT4_ARRAY),
            PythonDTO::PyFloat64(_) => Ok(tokio_postgres::types::Type::FLOAT8_ARRAY),
            PythonDTO::PyMoney(_) => Ok(tokio_postgres::types::Type::MONEY_ARRAY),
            PythonDTO::PyIpAddress(_) => Ok(tokio_postgres::types::Type::INET_ARRAY),
            PythonDTO::PyJsonb(_) => Ok(tokio_postgres::types::Type::JSONB_ARRAY),
            PythonDTO::PyJson(_) => Ok(tokio_postgres::types::Type::JSON_ARRAY),
            PythonDTO::PyDate(_) => Ok(tokio_postgres::types::Type::DATE_ARRAY),
            PythonDTO::PyTime(_) => Ok(tokio_postgres::types::Type::TIME_ARRAY),
            PythonDTO::PyDateTime(_) => Ok(tokio_postgres::types::Type::TIMESTAMP_ARRAY),
            PythonDTO::PyDateTimeTz(_) => Ok(tokio_postgres::types::Type::TIMESTAMPTZ_ARRAY),
            PythonDTO::PyMacAddr6(_) => Ok(tokio_postgres::types::Type::MACADDR_ARRAY),
            PythonDTO::PyMacAddr8(_) => Ok(tokio_postgres::types::Type::MACADDR8_ARRAY),
            PythonDTO::PyDecimal(_) => Ok(tokio_postgres::types::Type::NUMERIC_ARRAY),
            PythonDTO::PyPoint(_) => Ok(tokio_postgres::types::Type::POINT_ARRAY),
            PythonDTO::PyBox(_) => Ok(tokio_postgres::types::Type::BOX_ARRAY),
            PythonDTO::PyPath(_) => Ok(tokio_postgres::types::Type::PATH_ARRAY),
            PythonDTO::PyLine(_) => Ok(tokio_postgres::types::Type::LINE_ARRAY),
            PythonDTO::PyLineSegment(_) => Ok(tokio_postgres::types::Type::LSEG_ARRAY),
            PythonDTO::PyCircle(_) => Ok(tokio_postgres::types::Type::CIRCLE_ARRAY),
            PythonDTO::PyInterval(_) => Ok(tokio_postgres::types::Type::INTERVAL_ARRAY),
            _ => Err(RustPSQLDriverError::PyToRustValueConversionError(
                "Can't process array type, your type doesn't have support yet".into(),
            )),
        }
    }

    /// Convert enum into serde `Value`.
    ///
    /// # Errors
    /// May return Err Result if cannot convert python type into rust.
    pub fn to_serde_value(&self) -> RustPSQLDriverPyResult<Value> {
        match self {
            PythonDTO::PyNone => Ok(Value::Null),
            PythonDTO::PyBool(pybool) => Ok(json!(pybool)),
            PythonDTO::PyString(pystring)
            | PythonDTO::PyText(pystring)
            | PythonDTO::PyVarChar(pystring) => Ok(json!(pystring)),
            PythonDTO::PyIntI32(pyint) => Ok(json!(pyint)),
            PythonDTO::PyIntI64(pyint) => Ok(json!(pyint)),
            PythonDTO::PyIntU64(pyint) => Ok(json!(pyint)),
            PythonDTO::PyFloat32(pyfloat) => Ok(json!(pyfloat)),
            PythonDTO::PyFloat64(pyfloat) => Ok(json!(pyfloat)),
            PythonDTO::PyList(pylist) => {
                let mut vec_serde_values: Vec<Value> = vec![];

                for py_object in pylist {
                    vec_serde_values.push(py_object.to_serde_value()?);
                }

                Ok(json!(vec_serde_values))
            }
            PythonDTO::PyArray(array) => Ok(json!(pythondto_array_to_serde(Some(array.clone()))?)),
            PythonDTO::PyJsonb(py_dict) | PythonDTO::PyJson(py_dict) => Ok(py_dict.clone()),
            _ => Err(RustPSQLDriverError::PyToRustValueConversionError(
                "Cannot convert your type into Rust type".into(),
            )),
        }
    }
}

/// Implement `ToSql` trait.
///
/// It allows us to pass `PythonDTO` enum as parameter
/// directly into `.execute()` method in
/// `DatabasePool`, `Connection` and `Transaction`.
impl ToSql for PythonDTO {
    /// Answer the question Is this type can be passed into sql?
    ///
    /// Always True.
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    /// Convert our `PythonDTO` enum into bytes.
    ///
    /// We convert every inner type of `PythonDTO` enum variant
    /// into bytes and write them into bytes buffer.
    ///
    /// # Errors
    ///
    /// May return Err Result if cannot write bytes into buffer.
    #[allow(clippy::too_many_lines)]
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let mut return_is_null_true: bool = false;
        if *self == PythonDTO::PyNone {
            return_is_null_true = true;
        }

        match self {
            PythonDTO::PyNone => {}
            PythonDTO::PyCustomType(some_bytes) => {
                <&[u8] as ToSql>::to_sql(&some_bytes.as_slice(), ty, out)?;
            }
            PythonDTO::PyBytes(pybytes) => {
                <Vec<u8> as ToSql>::to_sql(pybytes, ty, out)?;
            }
            PythonDTO::PyBool(boolean) => types::bool_to_sql(*boolean, out),
            PythonDTO::PyVarChar(string) => {
                <&str as ToSql>::to_sql(&string.as_str(), ty, out)?;
            }
            PythonDTO::PyText(string) => {
                <&str as ToSql>::to_sql(&string.as_str(), ty, out)?;
            }
            PythonDTO::PyUUID(pyuuid) => {
                <Uuid as ToSql>::to_sql(pyuuid, ty, out)?;
            }
            PythonDTO::PyString(string) => {
                <&str as ToSql>::to_sql(&string.as_str(), ty, out)?;
            }
            PythonDTO::PyIntI16(int) => out.put_i16(*int),
            PythonDTO::PyIntI32(int) => out.put_i32(*int),
            PythonDTO::PyIntI64(int) | PythonDTO::PyMoney(int) => out.put_i64(*int),
            PythonDTO::PyIntU32(int) => out.put_u32(*int),
            PythonDTO::PyIntU64(int) => out.put_u64(*int),
            PythonDTO::PyFloat32(float) => out.put_f32(*float),
            PythonDTO::PyFloat64(float) => out.put_f64(*float),
            PythonDTO::PyDate(pydate) => {
                <&NaiveDate as ToSql>::to_sql(&pydate, ty, out)?;
            }
            PythonDTO::PyTime(pytime) => {
                <&NaiveTime as ToSql>::to_sql(&pytime, ty, out)?;
            }
            PythonDTO::PyDateTime(pydatetime_no_tz) => {
                <&NaiveDateTime as ToSql>::to_sql(&pydatetime_no_tz, ty, out)?;
            }
            PythonDTO::PyDateTimeTz(pydatetime_tz) => {
                <&DateTime<FixedOffset> as ToSql>::to_sql(&pydatetime_tz, ty, out)?;
            }
            PythonDTO::PyInterval(pyinterval) => {
                <&Interval as ToSql>::to_sql(&pyinterval, ty, out)?;
            }
            PythonDTO::PyIpAddress(pyidaddress) => {
                <&IpAddr as ToSql>::to_sql(&pyidaddress, ty, out)?;
            }
            PythonDTO::PyMacAddr6(pymacaddr) => {
                <&[u8] as ToSql>::to_sql(&pymacaddr.as_bytes(), ty, out)?;
            }
            PythonDTO::PyMacAddr8(pymacaddr) => {
                <&[u8] as ToSql>::to_sql(&pymacaddr.as_bytes(), ty, out)?;
            }
            PythonDTO::PyPoint(pypoint) => {
                <&RustPoint as ToSql>::to_sql(&&RustPoint::new(*pypoint), ty, out)?;
            }
            PythonDTO::PyBox(pybox) => {
                <&RustRect as ToSql>::to_sql(&&RustRect::new(*pybox), ty, out)?;
            }
            PythonDTO::PyPath(pypath) => {
                <&RustLineString as ToSql>::to_sql(&&RustLineString::new(pypath.clone()), ty, out)?;
            }
            PythonDTO::PyLine(pyline) => {
                <&Line as ToSql>::to_sql(&pyline, ty, out)?;
            }
            PythonDTO::PyLineSegment(pylinesegment) => {
                <&RustLineSegment as ToSql>::to_sql(
                    &&RustLineSegment::new(*pylinesegment),
                    ty,
                    out,
                )?;
            }
            PythonDTO::PyCircle(pycircle) => {
                <&Circle as ToSql>::to_sql(&pycircle, ty, out)?;
            }
            PythonDTO::PyList(py_iterable) | PythonDTO::PyTuple(py_iterable) => {
                let mut items = Vec::new();
                for inner in py_iterable {
                    items.push(inner);
                }
                if items.is_empty() {
                    return_is_null_true = true;
                } else {
                    items.to_sql(&items[0].array_type()?, out)?;
                }
            }
            PythonDTO::PyArray(array) => {
                if let Some(first_elem) = array.iter().nth(0) {
                    match first_elem.array_type() {
                        Ok(ok_type) => {
                            array.to_sql(&ok_type, out)?;
                        }
                        Err(_) => {
                            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                                "Cannot define array type.".into(),
                            ))?
                        }
                    }
                }
            }
            PythonDTO::PyJsonb(py_dict) | PythonDTO::PyJson(py_dict) => {
                <&Value as ToSql>::to_sql(&py_dict, ty, out)?;
            }
            PythonDTO::PyDecimal(py_decimal) => {
                <Decimal as ToSql>::to_sql(py_decimal, ty, out)?;
            }
            PythonDTO::PyBoolArray(array) => {
                array.to_sql(&Type::BOOL_ARRAY, out)?;
            }
            PythonDTO::PyUuidArray(array) => {
                array.to_sql(&Type::UUID_ARRAY, out)?;
            }
            PythonDTO::PyVarCharArray(array) => {
                array.to_sql(&Type::VARCHAR_ARRAY, out)?;
            }
            PythonDTO::PyTextArray(array) => {
                array.to_sql(&Type::TEXT_ARRAY, out)?;
            }
            PythonDTO::PyInt16Array(array) => {
                array.to_sql(&Type::INT2_ARRAY, out)?;
            }
            PythonDTO::PyInt32Array(array) => {
                array.to_sql(&Type::INT4_ARRAY, out)?;
            }
            PythonDTO::PyInt64Array(array) => {
                array.to_sql(&Type::INT8_ARRAY, out)?;
            }
            PythonDTO::PyFloat32Array(array) => {
                array.to_sql(&Type::FLOAT4, out)?;
            }
            PythonDTO::PyFloat64Array(array) => {
                array.to_sql(&Type::FLOAT8_ARRAY, out)?;
            }
            PythonDTO::PyMoneyArray(array) => {
                array.to_sql(&Type::MONEY_ARRAY, out)?;
            }
            PythonDTO::PyIpAddressArray(array) => {
                array.to_sql(&Type::INET_ARRAY, out)?;
            }
            PythonDTO::PyJSONBArray(array) => {
                array.to_sql(&Type::JSONB_ARRAY, out)?;
            }
            PythonDTO::PyJSONArray(array) => {
                array.to_sql(&Type::JSON_ARRAY, out)?;
            }
            PythonDTO::PyDateArray(array) => {
                array.to_sql(&Type::DATE_ARRAY, out)?;
            }
            PythonDTO::PyTimeArray(array) => {
                array.to_sql(&Type::TIME_ARRAY, out)?;
            }
            PythonDTO::PyDateTimeArray(array) => {
                array.to_sql(&Type::TIMESTAMP_ARRAY, out)?;
            }
            PythonDTO::PyDateTimeTZArray(array) => {
                array.to_sql(&Type::TIMESTAMPTZ_ARRAY, out)?;
            }
            PythonDTO::PyMacAddr6Array(array) => {
                array.to_sql(&Type::MACADDR_ARRAY, out)?;
            }
            PythonDTO::PyMacAddr8Array(array) => {
                array.to_sql(&Type::MACADDR8_ARRAY, out)?;
            }
            PythonDTO::PyNumericArray(array) => {
                array.to_sql(&Type::NUMERIC_ARRAY, out)?;
            }
            PythonDTO::PyPointArray(array) => {
                array.to_sql(&Type::POINT_ARRAY, out)?;
            }
            PythonDTO::PyBoxArray(array) => {
                array.to_sql(&Type::BOX_ARRAY, out)?;
            }
            PythonDTO::PyPathArray(array) => {
                array.to_sql(&Type::PATH_ARRAY, out)?;
            }
            PythonDTO::PyLineArray(array) => {
                array.to_sql(&Type::LINE_ARRAY, out)?;
            }
            PythonDTO::PyLsegArray(array) => {
                array.to_sql(&Type::LSEG_ARRAY, out)?;
            }
            PythonDTO::PyCircleArray(array) => {
                array.to_sql(&Type::CIRCLE_ARRAY, out)?;
            }
            PythonDTO::PyIntervalArray(array) => {
                array.to_sql(&Type::INTERVAL_ARRAY, out)?;
            }
            PythonDTO::PyPgVector(vector) => {
                <PgVector as ToSql>::to_sql(&PgVector::from(vector.clone()), ty, out)?;
            }
        }

        if return_is_null_true {
            Ok(tokio_postgres::types::IsNull::Yes)
        } else {
            Ok(tokio_postgres::types::IsNull::No)
        }
    }

    to_sql_checked!();
}

fn parse_kwargs_qs(querystring: &str) -> (String, Vec<String>) {
    let re = regex::Regex::new(r"\$\(([^)]+)\)p").unwrap();

    {
        let kq_read = KWARGS_QUERYSTRINGS.read().unwrap();
        let qs = kq_read.get(querystring);

        if let Some(qs) = qs {
            return qs.clone();
        }
    };

    let mut counter = 0;
    let mut sequence = Vec::new();

    let result = re.replace_all(querystring, |caps: &regex::Captures| {
        let account_id = caps[1].to_string();

        sequence.push(account_id.clone());
        counter += 1;

        format!("${}", &counter)
    });

    let mut kq_write = KWARGS_QUERYSTRINGS.write().unwrap();
    kq_write.insert(
        querystring.to_string(),
        (result.clone().into(), sequence.clone()),
    );
    (result.into(), sequence)
}

pub fn convert_kwargs_parameters<'a>(
    kw_params: &Bound<'_, PyMapping>,
    querystring: &'a str,
) -> RustPSQLDriverPyResult<(String, Vec<PythonDTO>)> {
    let mut result_vec: Vec<PythonDTO> = vec![];
    let (changed_string, params_names) = parse_kwargs_qs(querystring);

    for param_name in params_names {
        match kw_params.get_item(&param_name) {
            Ok(param) => result_vec.push(py_to_rust(&param)?),
            Err(_) => {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    format!("Cannot find parameter with name <{param_name}> in parameters").into(),
                ))
            }
        }
    }

    Ok((changed_string, result_vec))
}

pub fn convert_seq_parameters(
    seq_params: Vec<Py<PyAny>>,
) -> RustPSQLDriverPyResult<Vec<PythonDTO>> {
    let mut result_vec: Vec<PythonDTO> = vec![];
    Python::with_gil(|gil| {
        for parameter in seq_params {
            result_vec.push(py_to_rust(parameter.bind(gil))?);
        }
        Ok::<(), RustPSQLDriverError>(())
    })?;

    Ok(result_vec)
}

/// Convert parameters come from python.
///
/// Parameters for `execute()` method can be either
/// a list or a tuple or a set.
///
/// We parse every parameter from python object and return
/// Vector of out `PythonDTO`.
///
/// # Errors
///
/// May return Err Result if can't convert python object.
#[allow(clippy::needless_pass_by_value)]
pub fn convert_parameters_and_qs(
    querystring: String,
    parameters: Option<Py<PyAny>>,
) -> RustPSQLDriverPyResult<(String, Vec<PythonDTO>)> {
    let Some(parameters) = parameters else {
        return Ok((querystring, vec![]));
    };

    let res = Python::with_gil(|gil| {
        let params = parameters.extract::<Vec<Py<PyAny>>>(gil).map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError(
                "Cannot convert you parameters argument into Rust type, please use List/Tuple"
                    .into(),
            )
        });
        if let Ok(params) = params {
            return Ok((querystring, convert_seq_parameters(params)?));
        }

        let kw_params = parameters.downcast_bound::<PyMapping>(gil);
        if let Ok(kw_params) = kw_params {
            return convert_kwargs_parameters(kw_params, &querystring);
        }

        Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Parameters must be sequence or mapping".into(),
        ))
    })?;

    Ok(res)
}

/// Convert Sequence from Python (except String) into flat vec.
///
/// # Errors
/// May return Err Result if cannot convert element into Rust one.
pub fn py_sequence_into_flat_vec(
    parameter: &Bound<PyAny>,
) -> RustPSQLDriverPyResult<Vec<PythonDTO>> {
    let py_seq = parameter.downcast::<PySequence>().map_err(|_| {
        RustPSQLDriverError::PyToRustValueConversionError(
            "PostgreSQL ARRAY type can be made only from python Sequence".into(),
        )
    })?;

    let mut final_vec: Vec<PythonDTO> = vec![];

    for seq_elem in py_seq.iter()? {
        let ok_seq_elem = seq_elem?;

        // Check for the string because it's sequence too,
        // and in the most cases it should be array type, not new dimension.
        if ok_seq_elem.is_instance_of::<PyString>() {
            final_vec.push(py_to_rust(&ok_seq_elem)?);
            continue;
        }

        let possible_next_seq = ok_seq_elem.downcast::<PySequence>();

        if let Ok(next_seq) = possible_next_seq {
            let mut next_vec = py_sequence_into_flat_vec(next_seq)?;
            final_vec.append(&mut next_vec);
        } else {
            final_vec.push(py_to_rust(&ok_seq_elem)?);
            continue;
        }
    }

    Ok(final_vec)
}

/// Convert Sequence from Python into Postgres ARRAY.
///
/// # Errors
///
/// May return Err Result if cannot convert at least one element.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub fn py_sequence_into_postgres_array(
    parameter: &Bound<PyAny>,
) -> RustPSQLDriverPyResult<Array<PythonDTO>> {
    let mut py_seq = parameter
        .downcast::<PySequence>()
        .map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError(
                "PostgreSQL ARRAY type can be made only from python Sequence".into(),
            )
        })?
        .clone();

    let mut dimensions: Vec<Dimension> = vec![];
    let mut continue_iteration = true;

    while continue_iteration {
        dimensions.push(Dimension {
            len: py_seq.len()? as i32,
            lower_bound: 1,
        });

        let first_seq_elem = py_seq.iter()?.next();
        match first_seq_elem {
            Some(first_seq_elem) => {
                if let Ok(first_seq_elem) = first_seq_elem {
                    // Check for the string because it's sequence too,
                    // and in the most cases it should be array type, not new dimension.
                    if first_seq_elem.is_instance_of::<PyString>() {
                        continue_iteration = false;
                        continue;
                    }
                    let possible_inner_seq = first_seq_elem.downcast::<PySequence>();

                    match possible_inner_seq {
                        Ok(possible_inner_seq) => {
                            py_seq = possible_inner_seq.clone();
                        }
                        Err(_) => continue_iteration = false,
                    }
                }
            }
            None => {
                continue_iteration = false;
            }
        }
    }

    let array_data = py_sequence_into_flat_vec(parameter)?;
    match postgres_array::Array::from_parts_no_panic(array_data, dimensions) {
        Ok(result_array) => Ok(result_array),
        Err(err) => Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
            "Cannot convert python sequence to PostgreSQL ARRAY, error - {err}"
        ))),
    }
}

/// Extract a value from a Python object, raising an error if missing or invalid
///
/// # Errors
/// This function will return `Err` in the following cases:
/// - The Python object does not have the specified attribute
/// - The attribute exists but cannot be extracted into the specified Rust type
fn extract_value_from_python_object_or_raise<'py, T>(
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

/// Extract a timezone-aware datetime from a Python object.
/// This function retrieves various datetime components (`year`, `month`, `day`, etc.)
/// from a Python object and constructs a `DateTime<FixedOffset>`
///
/// # Errors
/// This function will return `Err` in the following cases:
/// - The Python object does not contain or support one or more required datetime attributes
/// - The retrieved values are invalid for constructing a date, time, or datetime (e.g., invalid month or day)
/// - The timezone information (`tzinfo`) is not available or cannot be parsed
/// - The resulting datetime is ambiguous or invalid (e.g., due to DST transitions)
fn extract_datetime_from_python_object_attrs(
    parameter: &pyo3::Bound<'_, PyAny>,
) -> Result<DateTime<FixedOffset>, RustPSQLDriverError> {
    let year = extract_value_from_python_object_or_raise::<i32>(parameter, "year")?;
    let month = extract_value_from_python_object_or_raise::<u32>(parameter, "month")?;
    let day = extract_value_from_python_object_or_raise::<u32>(parameter, "day")?;
    let hour = extract_value_from_python_object_or_raise::<u32>(parameter, "hour")?;
    let minute = extract_value_from_python_object_or_raise::<u32>(parameter, "minute")?;
    let second = extract_value_from_python_object_or_raise::<u32>(parameter, "second")?;
    let microsecond = extract_value_from_python_object_or_raise::<u32>(parameter, "microsecond")?;

    let date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| RustPSQLDriverError::PyToRustValueConversionError("Invalid date".into()))?;
    let time = NaiveTime::from_hms_micro_opt(hour, minute, second, microsecond)
        .ok_or_else(|| RustPSQLDriverError::PyToRustValueConversionError("Invalid time".into()))?;
    let naive_datetime = NaiveDateTime::new(date, time);

    let raw_timestamp_tz = parameter
        .getattr("tzinfo")
        .ok()
        .and_then(|tzinfo| tzinfo.getattr("key").ok())
        .and_then(|key| key.extract::<String>().ok())
        .ok_or_else(|| {
            RustPSQLDriverError::PyToRustValueConversionError("Invalid timezone info".into())
        })?;

    let fixed_offset_datetime = raw_timestamp_tz
        .parse::<Tz>()
        .map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError("Failed to parse TZ".into())
        })?
        .from_local_datetime(&naive_datetime)
        .single()
        .ok_or_else(|| {
            RustPSQLDriverError::PyToRustValueConversionError(
                "Ambiguous or invalid datetime".into(),
            )
        })?
        .fixed_offset();

    Ok(fixed_offset_datetime)
}

/// Convert single python parameter to `PythonDTO` enum.
///
/// # Errors
///
/// May return Err Result if python type doesn't have support yet
/// or value of the type is incorrect.
#[allow(clippy::too_many_lines)]
pub fn py_to_rust(parameter: &pyo3::Bound<'_, PyAny>) -> RustPSQLDriverPyResult<PythonDTO> {
    if parameter.is_none() {
        return Ok(PythonDTO::PyNone);
    }

    if parameter.is_instance_of::<extra_types::CustomType>() {
        return Ok(PythonDTO::PyCustomType(
            parameter.extract::<extra_types::CustomType>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<PyBool>() {
        return Ok(PythonDTO::PyBool(parameter.extract::<bool>()?));
    }

    if parameter.is_instance_of::<PyBytes>() {
        return Ok(PythonDTO::PyBytes(parameter.extract::<Vec<u8>>()?));
    }

    if parameter.is_instance_of::<extra_types::Text>() {
        return Ok(PythonDTO::PyText(
            parameter.extract::<extra_types::Text>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<extra_types::VarChar>() {
        return Ok(PythonDTO::PyVarChar(
            parameter.extract::<extra_types::VarChar>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<PyString>() {
        return Ok(PythonDTO::PyString(parameter.extract::<String>()?));
    }

    if parameter.is_instance_of::<PyFloat>() {
        return Ok(PythonDTO::PyFloat64(parameter.extract::<f64>()?));
    }

    if parameter.is_instance_of::<extra_types::Float32>() {
        return Ok(PythonDTO::PyFloat32(
            parameter
                .extract::<extra_types::Float32>()?
                .retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Float64>() {
        return Ok(PythonDTO::PyFloat64(
            parameter
                .extract::<extra_types::Float64>()?
                .retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::SmallInt>() {
        return Ok(PythonDTO::PyIntI16(
            parameter
                .extract::<extra_types::SmallInt>()?
                .retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Integer>() {
        return Ok(PythonDTO::PyIntI32(
            parameter
                .extract::<extra_types::Integer>()?
                .retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::BigInt>() {
        return Ok(PythonDTO::PyIntI64(
            parameter.extract::<extra_types::BigInt>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Money>() {
        return Ok(PythonDTO::PyMoney(
            parameter.extract::<extra_types::Money>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<PyInt>() {
        return Ok(PythonDTO::PyIntI32(parameter.extract::<i32>()?));
    }

    if parameter.is_instance_of::<PyDateTime>() {
        let timestamp_tz = parameter.extract::<DateTime<FixedOffset>>();
        if let Ok(pydatetime_tz) = timestamp_tz {
            return Ok(PythonDTO::PyDateTimeTz(pydatetime_tz));
        }

        let timestamp_no_tz = parameter.extract::<NaiveDateTime>();
        if let Ok(pydatetime_no_tz) = timestamp_no_tz {
            return Ok(PythonDTO::PyDateTime(pydatetime_no_tz));
        }

        let timestamp_tz = extract_datetime_from_python_object_attrs(parameter);
        if let Ok(pydatetime_tz) = timestamp_tz {
            return Ok(PythonDTO::PyDateTimeTz(pydatetime_tz));
        }

        return Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Can not convert you datetime to rust type".into(),
        ));
    }

    if parameter.is_instance_of::<PyDate>() {
        return Ok(PythonDTO::PyDate(parameter.extract::<NaiveDate>()?));
    }

    if parameter.is_instance_of::<PyTime>() {
        return Ok(PythonDTO::PyTime(parameter.extract::<NaiveTime>()?));
    }

    if parameter.is_instance_of::<PyDelta>() {
        let duration = parameter.extract::<chrono::Duration>()?;
        if let Some(interval) = Interval::from_duration(duration) {
            return Ok(PythonDTO::PyInterval(interval));
        }
        return Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Cannot convert timedelta from Python to inner Rust type.".to_string(),
        ));
    }

    if parameter.is_instance_of::<PyList>() | parameter.is_instance_of::<PyTuple>() {
        return Ok(PythonDTO::PyArray(py_sequence_into_postgres_array(
            parameter,
        )?));
    }

    if parameter.is_instance_of::<PyDict>() {
        let dict = parameter.downcast::<PyDict>().map_err(|error| {
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
            let value = py_to_rust(&py_list.get_item(1)?)?;

            serde_map.insert(key, value.to_serde_value()?);
        }

        return Ok(PythonDTO::PyJsonb(Value::Object(serde_map)));
    }

    if parameter.is_instance_of::<extra_types::JSONB>() {
        return Ok(PythonDTO::PyJsonb(
            parameter.extract::<extra_types::JSONB>()?.inner().clone(),
        ));
    }

    if parameter.is_instance_of::<extra_types::JSON>() {
        return Ok(PythonDTO::PyJson(
            parameter.extract::<extra_types::JSON>()?.inner().clone(),
        ));
    }

    if parameter.is_instance_of::<extra_types::MacAddr6>() {
        return Ok(PythonDTO::PyMacAddr6(
            parameter.extract::<extra_types::MacAddr6>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<extra_types::MacAddr8>() {
        return Ok(PythonDTO::PyMacAddr8(
            parameter.extract::<extra_types::MacAddr8>()?.inner(),
        ));
    }

    if parameter.get_type().name()? == "UUID" {
        return Ok(PythonDTO::PyUUID(Uuid::parse_str(
            parameter.str()?.extract::<&str>()?,
        )?));
    }

    if parameter.get_type().name()? == "decimal.Decimal"
        || parameter.get_type().name()? == "Decimal"
    {
        return Ok(PythonDTO::PyDecimal(Decimal::from_str_exact(
            parameter.str()?.extract::<&str>()?,
        )?));
    }

    if parameter.is_instance_of::<extra_types::Point>() {
        return Ok(PythonDTO::PyPoint(
            parameter.extract::<extra_types::Point>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Box>() {
        return Ok(PythonDTO::PyBox(
            parameter.extract::<extra_types::Box>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Path>() {
        return Ok(PythonDTO::PyPath(
            parameter.extract::<extra_types::Path>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Line>() {
        return Ok(PythonDTO::PyLine(
            parameter.extract::<extra_types::Line>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::LineSegment>() {
        return Ok(PythonDTO::PyLineSegment(
            parameter
                .extract::<extra_types::LineSegment>()?
                .retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::Circle>() {
        return Ok(PythonDTO::PyCircle(
            parameter.extract::<extra_types::Circle>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<extra_types::BoolArray>() {
        return parameter
            .extract::<extra_types::BoolArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::UUIDArray>() {
        return parameter
            .extract::<extra_types::UUIDArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::VarCharArray>() {
        return parameter
            .extract::<extra_types::VarCharArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::TextArray>() {
        return parameter
            .extract::<extra_types::TextArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::Int16Array>() {
        return parameter
            .extract::<extra_types::Int16Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::Int32Array>() {
        return parameter
            .extract::<extra_types::Int32Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::Int64Array>() {
        return parameter
            .extract::<extra_types::Int64Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::Float32Array>() {
        return parameter
            .extract::<extra_types::Float32Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::Float64Array>() {
        return parameter
            .extract::<extra_types::Float64Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::MoneyArray>() {
        return parameter
            .extract::<extra_types::MoneyArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::IpAddressArray>() {
        return parameter
            .extract::<extra_types::IpAddressArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::JSONBArray>() {
        return parameter
            .extract::<extra_types::JSONBArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::JSONArray>() {
        return parameter
            .extract::<extra_types::JSONArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::DateArray>() {
        return parameter
            .extract::<extra_types::DateArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::TimeArray>() {
        return parameter
            .extract::<extra_types::TimeArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::DateTimeArray>() {
        return parameter
            .extract::<extra_types::DateTimeArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::DateTimeTZArray>() {
        return parameter
            .extract::<extra_types::DateTimeTZArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::MacAddr6Array>() {
        return parameter
            .extract::<extra_types::MacAddr6Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::MacAddr8Array>() {
        return parameter
            .extract::<extra_types::MacAddr8Array>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::NumericArray>() {
        return parameter
            .extract::<extra_types::NumericArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::PointArray>() {
        return parameter
            .extract::<extra_types::PointArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::BoxArray>() {
        return parameter
            .extract::<extra_types::BoxArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::PathArray>() {
        return parameter
            .extract::<extra_types::PathArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::LineArray>() {
        return parameter
            .extract::<extra_types::LineArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::LsegArray>() {
        return parameter
            .extract::<extra_types::LsegArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::CircleArray>() {
        return parameter
            .extract::<extra_types::CircleArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::IntervalArray>() {
        return parameter
            .extract::<extra_types::IntervalArray>()?
            ._convert_to_python_dto();
    }

    if parameter.is_instance_of::<extra_types::PgVector>() {
        return Ok(PythonDTO::PyPgVector(
            parameter.extract::<extra_types::PgVector>()?.inner_value(),
        ));
    }

    if let Ok(id_address) = parameter.extract::<IpAddr>() {
        return Ok(PythonDTO::PyIpAddress(id_address));
    }

    // It's used for Enum.
    // If StrEnum is used on Python side,
    // we simply stop at the `is_instance_of::<PyString>``.
    if let Ok(value_attr) = parameter.getattr("value") {
        if let Ok(possible_string) = value_attr.extract::<String>() {
            return Ok(PythonDTO::PyString(possible_string));
        }
    }

    Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
        "Can not covert you type {parameter} into inner one",
    )))
}

fn composite_field_postgres_to_py<'a, T: FromSql<'a>>(
    type_: &Type,
    buf: &mut &'a [u8],
    is_simple: bool,
) -> RustPSQLDriverPyResult<T> {
    if is_simple {
        return T::from_sql_nullable(type_, Some(buf)).map_err(|err| {
            RustPSQLDriverError::RustToPyValueConversionError(format!(
                "Cannot convert PostgreSQL type {type_} into Python type, err: {err}",
            ))
        });
    }
    postgres_types::private::read_value::<T>(type_, buf).map_err(|err| {
        RustPSQLDriverError::RustToPyValueConversionError(format!(
            "Cannot convert PostgreSQL type {type_} into Python type, err: {err}",
        ))
    })
}

/// Convert Array of `PythonDTO`s to serde `Value`.
///
/// It can convert multidimensional arrays.
fn pythondto_array_to_serde(array: Option<Array<PythonDTO>>) -> RustPSQLDriverPyResult<Value> {
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
) -> RustPSQLDriverPyResult<Value> {
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

/// Convert rust array to python list.
///
/// It can convert multidimensional arrays.
fn postgres_array_to_py<T: ToPyObject>(
    py: Python<'_>,
    array: Option<Array<T>>,
) -> Option<Py<PyList>> {
    array.map(|array| {
        inner_postgres_array_to_py(
            py,
            array.dimensions(),
            array.iter().collect::<Vec<&T>>().as_slice(),
            0,
            0,
        )
    })
}

/// Inner postgres array conversion to python list.
#[allow(clippy::cast_sign_loss)]
fn inner_postgres_array_to_py<T>(
    py: Python<'_>,
    dimensions: &[Dimension],
    data: &[T],
    dimension_index: usize,
    mut lower_bound: usize,
) -> Py<PyList>
where
    T: ToPyObject,
{
    let current_dimension = dimensions.get(dimension_index);

    if let Some(current_dimension) = current_dimension {
        let possible_next_dimension = dimensions.get(dimension_index + 1);
        match possible_next_dimension {
            Some(next_dimension) => {
                let final_list = PyList::empty_bound(py);

                for _ in 0..current_dimension.len as usize {
                    if dimensions.get(dimension_index + 1).is_some() {
                        let inner_pylist = inner_postgres_array_to_py(
                            py,
                            dimensions,
                            &data[lower_bound..next_dimension.len as usize + lower_bound],
                            dimension_index + 1,
                            0,
                        );
                        final_list.append(inner_pylist).unwrap();
                        lower_bound += next_dimension.len as usize;
                    };
                }

                return final_list.unbind();
            }
            None => {
                return PyList::new_bound(py, data).unbind();
            }
        }
    }

    PyList::empty_bound(py).unbind()
}

#[allow(clippy::too_many_lines)]
fn postgres_bytes_to_py(
    py: Python<'_>,
    type_: &Type,
    buf: &mut &[u8],
    is_simple: bool,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    match *type_ {
        // ---------- Bytes Types ----------
        // Convert BYTEA type into Vector<u8>, then into PyBytes
        Type::BYTEA => {
            let vec_of_bytes =
                composite_field_postgres_to_py::<Option<Vec<u8>>>(type_, buf, is_simple)?;
            if let Some(vec_of_bytes) = vec_of_bytes {
                return Ok(PyBytes::new_bound(py, &vec_of_bytes).to_object(py));
            }
            Ok(py.None())
        }
        // // ---------- String Types ----------
        // // Convert TEXT and VARCHAR type into String, then into str
        Type::TEXT | Type::VARCHAR | Type::XML => Ok(composite_field_postgres_to_py::<
            Option<String>,
        >(type_, buf, is_simple)?
        .to_object(py)),
        // ---------- Boolean Types ----------
        // Convert BOOL type into bool
        Type::BOOL => Ok(
            composite_field_postgres_to_py::<Option<bool>>(type_, buf, is_simple)?.to_object(py),
        ),
        // ---------- Number Types ----------
        // Convert SmallInt into i16, then into int
        Type::INT2 => {
            Ok(composite_field_postgres_to_py::<Option<i16>>(type_, buf, is_simple)?.to_object(py))
        }
        // Convert Integer into i32, then into int
        Type::INT4 => {
            Ok(composite_field_postgres_to_py::<Option<i32>>(type_, buf, is_simple)?.to_object(py))
        }
        // Convert BigInt into i64, then into int
        Type::INT8 | Type::MONEY => {
            Ok(composite_field_postgres_to_py::<Option<i64>>(type_, buf, is_simple)?.to_object(py))
        }
        // Convert REAL into f32, then into float
        Type::FLOAT4 => {
            Ok(composite_field_postgres_to_py::<Option<f32>>(type_, buf, is_simple)?.to_object(py))
        }
        // Convert DOUBLE PRECISION into f64, then into float
        Type::FLOAT8 => {
            Ok(composite_field_postgres_to_py::<Option<f64>>(type_, buf, is_simple)?.to_object(py))
        }
        // ---------- Date Types ----------
        // Convert DATE into NaiveDate, then into datetime.date
        Type::DATE => Ok(composite_field_postgres_to_py::<Option<NaiveDate>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert Time into NaiveTime, then into datetime.time
        Type::TIME => Ok(composite_field_postgres_to_py::<Option<NaiveTime>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMP => Ok(composite_field_postgres_to_py::<Option<NaiveDateTime>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMPTZ => Ok(
            composite_field_postgres_to_py::<Option<DateTime<FixedOffset>>>(type_, buf, is_simple)?
                .to_object(py),
        ),
        // ---------- UUID Types ----------
        // Convert UUID into Uuid type, then into String if possible
        Type::UUID => {
            let rust_uuid = composite_field_postgres_to_py::<Option<Uuid>>(type_, buf, is_simple)?;
            match rust_uuid {
                Some(rust_uuid) => {
                    return Ok(PyString::new_bound(py, &rust_uuid.to_string()).to_object(py))
                }
                None => Ok(py.None()),
            }
        }
        // ---------- IpAddress Types ----------
        Type::INET => Ok(
            composite_field_postgres_to_py::<Option<IpAddr>>(type_, buf, is_simple)?.to_object(py),
        ),
        // Convert JSON/JSONB into Serde Value, then into list or dict
        Type::JSONB | Type::JSON => {
            let db_json = composite_field_postgres_to_py::<Option<Value>>(type_, buf, is_simple)?;

            match db_json {
                Some(value) => Ok(build_python_from_serde_value(py, value)?),
                None => Ok(py.None().to_object(py)),
            }
        }
        // Convert MACADDR into inner type for macaddr6, then into str
        Type::MACADDR => {
            let macaddr_ =
                composite_field_postgres_to_py::<Option<RustMacAddr6>>(type_, buf, is_simple)?;
            if let Some(macaddr_) = macaddr_ {
                Ok(macaddr_.inner().to_string().to_object(py))
            } else {
                Ok(py.None().to_object(py))
            }
        }
        Type::MACADDR8 => {
            let macaddr_ =
                composite_field_postgres_to_py::<Option<RustMacAddr8>>(type_, buf, is_simple)?;
            if let Some(macaddr_) = macaddr_ {
                Ok(macaddr_.inner().to_string().to_object(py))
            } else {
                Ok(py.None().to_object(py))
            }
        }
        Type::NUMERIC => {
            if let Some(numeric_) =
                composite_field_postgres_to_py::<Option<Decimal>>(type_, buf, is_simple)?
            {
                return Ok(InnerDecimal(numeric_).to_object(py));
            }
            Ok(py.None().to_object(py))
        }
        // ---------- Geo Types ----------
        Type::POINT => {
            let point_ =
                composite_field_postgres_to_py::<Option<RustPoint>>(type_, buf, is_simple)?;

            match point_ {
                Some(point_) => Ok(point_.into_py(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::BOX => {
            let box_ = composite_field_postgres_to_py::<Option<RustRect>>(type_, buf, is_simple)?;

            match box_ {
                Some(box_) => Ok(box_.into_py(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::PATH => {
            let path_ =
                composite_field_postgres_to_py::<Option<RustLineString>>(type_, buf, is_simple)?;

            match path_ {
                Some(path_) => Ok(path_.into_py(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::LINE => {
            let line_ = composite_field_postgres_to_py::<Option<Line>>(type_, buf, is_simple)?;

            match line_ {
                Some(line_) => Ok(line_.into_py(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::LSEG => {
            let lseg_ =
                composite_field_postgres_to_py::<Option<RustLineSegment>>(type_, buf, is_simple)?;

            match lseg_ {
                Some(lseg_) => Ok(lseg_.into_py(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::CIRCLE => {
            let circle_ = composite_field_postgres_to_py::<Option<Circle>>(type_, buf, is_simple)?;

            match circle_ {
                Some(circle_) => Ok(circle_.into_py(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::INTERVAL => {
            let interval =
                composite_field_postgres_to_py::<Option<Interval>>(type_, buf, is_simple)?;
            if let Some(interval) = interval {
                return Ok(InnerInterval(interval).to_object(py));
            }
            Ok(py.None())
        }
        // ---------- Array Text Types ----------
        Type::BOOL_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<bool>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of TEXT or VARCHAR into Vec<String>, then into list[str]
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::XML_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<String>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // ---------- Array Integer Types ----------
        // Convert ARRAY of SmallInt into Vec<i16>, then into list[int]
        Type::INT2_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i16>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of Integer into Vec<i32>, then into list[int]
        Type::INT4_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i32>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of BigInt into Vec<i64>, then into list[int]
        Type::INT8_ARRAY | Type::MONEY_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i64>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of Float4 into Vec<f32>, then into list[float]
        Type::FLOAT4_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<f32>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of Float8 into Vec<f64>, then into list[float]
        Type::FLOAT8_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<f64>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of Date into Vec<NaiveDate>, then into list[datetime.date]
        Type::DATE_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<NaiveDate>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of Time into Vec<NaiveTime>, then into list[datetime.date]
        Type::TIME_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<NaiveTime>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of TIMESTAMP into Vec<NaiveDateTime>, then into list[datetime.date]
        Type::TIMESTAMP_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<NaiveDateTime>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // Convert ARRAY of TIMESTAMPTZ into Vec<DateTime<FixedOffset>>, then into list[datetime.date]
        Type::TIMESTAMPTZ_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<DateTime<FixedOffset>>>>(
                type_, buf, is_simple,
            )?,
        )
        .to_object(py)),
        // Convert ARRAY of UUID into Vec<Array<InternalUuid>>, then into list[UUID]
        Type::UUID_ARRAY => {
            let uuid_array = composite_field_postgres_to_py::<Option<Array<InternalUuid>>>(
                type_, buf, is_simple,
            )?;
            Ok(postgres_array_to_py(py, uuid_array).to_object(py))
        }
        // Convert ARRAY of INET into Vec<INET>, then into list[IPv4Address | IPv6Address]
        Type::INET_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<IpAddr>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        Type::JSONB_ARRAY | Type::JSON_ARRAY => {
            let db_json_array = composite_field_postgres_to_py::<Option<Array<InternalSerdeValue>>>(
                type_, buf, is_simple,
            )?;
            Ok(postgres_array_to_py(py, db_json_array).to_object(py))
        }
        Type::NUMERIC_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<InnerDecimal>>>(type_, buf, is_simple)?,
        )
        .to_object(py)),
        // ---------- Array Geo Types ----------
        Type::POINT_ARRAY => {
            let point_array_ =
                composite_field_postgres_to_py::<Option<Array<RustPoint>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, point_array_).to_object(py))
        }
        Type::BOX_ARRAY => {
            let box_array_ =
                composite_field_postgres_to_py::<Option<Array<RustRect>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, box_array_).to_object(py))
        }
        Type::PATH_ARRAY => {
            let path_array_ = composite_field_postgres_to_py::<Option<Array<RustLineString>>>(
                type_, buf, is_simple,
            )?;

            Ok(postgres_array_to_py(py, path_array_).to_object(py))
        }
        Type::LINE_ARRAY => {
            let line_array_ =
                composite_field_postgres_to_py::<Option<Array<Line>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, line_array_).to_object(py))
        }
        Type::LSEG_ARRAY => {
            let lseg_array_ = composite_field_postgres_to_py::<Option<Array<RustLineSegment>>>(
                type_, buf, is_simple,
            )?;

            Ok(postgres_array_to_py(py, lseg_array_).to_object(py))
        }
        Type::CIRCLE_ARRAY => {
            let circle_array_ =
                composite_field_postgres_to_py::<Option<Array<Circle>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, circle_array_).to_object(py))
        }
        Type::INTERVAL_ARRAY => {
            let interval_array_ = composite_field_postgres_to_py::<Option<Array<InnerInterval>>>(
                type_, buf, is_simple,
            )?;

            Ok(postgres_array_to_py(py, interval_array_).to_object(py))
        }
        _ => other_postgres_bytes_to_py(py, type_, buf, is_simple),
    }
}

/// Convert OTHER type to python.
///
/// # Errors
/// May return result if type is unknown.
pub fn other_postgres_bytes_to_py(
    py: Python<'_>,
    type_: &Type,
    buf: &mut &[u8],
    is_simple: bool,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    if type_.name() == "vector" {
        let vector = composite_field_postgres_to_py::<Option<PgVector>>(type_, buf, is_simple)?;
        match vector {
            Some(real_vector) => {
                return Ok(real_vector.to_vec().to_object(py));
            }
            None => return Ok(py.None()),
        }
    }

    Err(RustPSQLDriverError::RustToPyValueConversionError(
        format!("Cannot convert {type_} into Python type, please look at the custom_decoders functionality.")
    ))
}

/// Convert composite type from `PostgreSQL` to Python type.
///
/// # Errors
/// May return error if there is any problem with bytes.
#[allow(clippy::cast_sign_loss)]
pub fn composite_postgres_to_py(
    py: Python<'_>,
    fields: &Vec<Field>,
    buf: &mut &[u8],
    custom_decoders: &Option<Py<PyDict>>,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    let result_py_dict: Bound<'_, PyDict> = PyDict::new_bound(py);

    let num_fields = postgres_types::private::read_be_i32(buf).map_err(|err| {
        RustPSQLDriverError::RustToPyValueConversionError(format!(
            "Cannot read bytes data from PostgreSQL: {err}"
        ))
    })?;
    if num_fields as usize != fields.len() {
        return Err(RustPSQLDriverError::RustToPyValueConversionError(format!(
            "invalid field count: {} vs {}",
            num_fields,
            fields.len()
        )));
    }

    for field in fields {
        let oid = postgres_types::private::read_be_i32(buf).map_err(|err| {
            RustPSQLDriverError::RustToPyValueConversionError(format!(
                "Cannot read bytes data from PostgreSQL: {err}"
            ))
        })? as u32;

        if oid != field.type_().oid() {
            return Err(RustPSQLDriverError::RustToPyValueConversionError(
                "unexpected OID".into(),
            ));
        }

        match field.type_().kind() {
            Kind::Simple | Kind::Array(_) => {
                result_py_dict.set_item(
                    field.name(),
                    postgres_bytes_to_py(py, field.type_(), buf, false)?.to_object(py),
                )?;
            }
            Kind::Enum(_) => {
                result_py_dict.set_item(
                    field.name(),
                    postgres_bytes_to_py(py, &Type::VARCHAR, buf, false)?.to_object(py),
                )?;
            }
            _ => {
                let (_, tail) = buf.split_at(4_usize);
                *buf = tail;
                result_py_dict.set_item(
                    field.name(),
                    raw_bytes_data_process(py, buf, field.name(), field.type_(), custom_decoders)?
                        .to_object(py),
                )?;
            }
        }
    }

    Ok(result_py_dict.to_object(py))
}

/// Process raw bytes from `PostgreSQL`.
///
/// # Errors
///
/// May return Err Result if cannot convert postgres
/// type into rust one.
pub fn raw_bytes_data_process(
    py: Python<'_>,
    raw_bytes_data: &mut &[u8],
    column_name: &str,
    column_type: &Type,
    custom_decoders: &Option<Py<PyDict>>,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    if let Some(custom_decoders) = custom_decoders {
        let py_encoder_func = custom_decoders
            .bind(py)
            .get_item(column_name.to_lowercase());

        if let Ok(Some(py_encoder_func)) = py_encoder_func {
            return Ok(py_encoder_func
                .call((raw_bytes_data.to_vec(),), None)?
                .unbind());
        }
    }

    match column_type.kind() {
        Kind::Simple | Kind::Array(_) => {
            postgres_bytes_to_py(py, column_type, raw_bytes_data, true)
        }
        Kind::Composite(fields) => {
            composite_postgres_to_py(py, fields, raw_bytes_data, custom_decoders)
        }
        Kind::Enum(_) => postgres_bytes_to_py(py, &Type::VARCHAR, raw_bytes_data, true),
        _ => Err(RustPSQLDriverError::RustToPyValueConversionError(
            column_type.to_string(),
        )),
    }
}

/// Convert type from postgres to python type.
///
/// # Errors
///
/// May return Err Result if cannot convert postgres
/// type into rust one.
pub fn postgres_to_py(
    py: Python<'_>,
    row: &Row,
    column: &Column,
    column_i: usize,
    custom_decoders: &Option<Py<PyDict>>,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    let raw_bytes_data = row.col_buffer(column_i);
    if let Some(mut raw_bytes_data) = raw_bytes_data {
        return raw_bytes_data_process(
            py,
            &mut raw_bytes_data,
            column.name(),
            column.type_(),
            custom_decoders,
        );
    }
    Ok(py.None())
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

/// Convert serde `Value` into Python object.
/// # Errors
/// May return Err Result if cannot add new value to Python Dict.
pub fn build_python_from_serde_value(
    py: Python<'_>,
    value: Value,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    match value {
        Value::Array(massive) => {
            let mut result_vec: Vec<Py<PyAny>> = vec![];

            for single_record in massive {
                result_vec.push(build_python_from_serde_value(py, single_record)?);
            }

            Ok(result_vec.to_object(py))
        }
        Value::Object(mapping) => {
            let py_dict = PyDict::new_bound(py);

            for (key, value) in mapping {
                py_dict.set_item(
                    build_python_from_serde_value(py, Value::String(key))?,
                    build_python_from_serde_value(py, value)?,
                )?;
            }

            Ok(py_dict.to_object(py))
        }
        Value::Bool(boolean) => Ok(boolean.to_object(py)),
        Value::Number(number) => {
            if number.is_f64() {
                Ok(number.as_f64().to_object(py))
            } else if number.is_i64() {
                Ok(number.as_i64().to_object(py))
            } else {
                Ok(number.as_u64().to_object(py))
            }
        }
        Value::String(string) => Ok(string.to_object(py)),
        Value::Null => Ok(py.None()),
    }
}

/// Convert Python sequence to Rust vector.
/// Also it checks that sequence has set/list/tuple type.
///
/// # Errors
///
/// May return error if cannot convert Python type into Rust one.
/// May return error if parameters type isn't correct.
fn py_sequence_to_rust(bind_parameters: &Bound<PyAny>) -> RustPSQLDriverPyResult<Vec<Py<PyAny>>> {
    let mut coord_values_sequence_vec: Vec<Py<PyAny>> = vec![];

    if bind_parameters.is_instance_of::<PySet>() {
        let bind_pyset_parameters = bind_parameters.downcast::<PySet>().unwrap();

        for one_parameter in bind_pyset_parameters {
            let extracted_parameter = one_parameter.extract::<Py<PyAny>>().map_err(|_| {
                RustPSQLDriverError::PyToRustValueConversionError(
                    format!("Error on sequence type extraction, please use correct list/tuple/set, {bind_parameters}")
                )
            })?;
            coord_values_sequence_vec.push(extracted_parameter);
        }
    } else if bind_parameters.is_instance_of::<PyList>()
        | bind_parameters.is_instance_of::<PyTuple>()
    {
        coord_values_sequence_vec = bind_parameters.extract::<Vec<Py<PyAny>>>().map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError(
                format!("Error on sequence type extraction, please use correct list/tuple/set, {bind_parameters}")
            )
        })?;
    } else {
        return Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
            "Invalid sequence type, please use list/tuple/set, {bind_parameters}"
        )));
    };

    Ok::<Vec<Py<PyAny>>, RustPSQLDriverError>(coord_values_sequence_vec)
}

/// Convert two python parameters(x and y) to Coord from `geo_type`.
/// Also it checks that passed values is int or float.
///
/// # Errors
///
/// May return error if cannot convert Python type into Rust one.
/// May return error if parameters type isn't correct.
fn convert_py_to_rust_coord_values(parameters: Vec<Py<PyAny>>) -> RustPSQLDriverPyResult<Vec<f64>> {
    Python::with_gil(|gil| {
        let mut coord_values_vec: Vec<f64> = vec![];

        for one_parameter in parameters {
            let parameter_bind = one_parameter.bind(gil);

            if !parameter_bind.is_instance_of::<PyFloat>()
                & !parameter_bind.is_instance_of::<PyInt>()
            {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "Incorrect types of coordinate values. It must be int or float".into(),
                ));
            }

            let python_dto = py_to_rust(parameter_bind)?;
            match python_dto {
                PythonDTO::PyIntI16(pyint) => coord_values_vec.push(f64::from(pyint)),
                PythonDTO::PyIntI32(pyint) => coord_values_vec.push(f64::from(pyint)),
                PythonDTO::PyIntU32(pyint) => coord_values_vec.push(f64::from(pyint)),
                PythonDTO::PyFloat32(pyfloat) => coord_values_vec.push(f64::from(pyfloat)),
                PythonDTO::PyFloat64(pyfloat) => coord_values_vec.push(pyfloat),
                PythonDTO::PyIntI64(_) | PythonDTO::PyIntU64(_) => {
                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                        "Not implemented this type yet".into(),
                    ))
                }
                _ => {
                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                        "Incorrect types of coordinate values. It must be int or float".into(),
                    ))
                }
            };
        }

        Ok::<Vec<f64>, RustPSQLDriverError>(coord_values_vec)
    })
}

/// Convert Python values with coordinates into vector of Coord's for building Geo types later.
///
/// Passed parameter can be either a list or a tuple or a set.
/// Inside this parameter may be multiple list/tuple/set with int/float or only int/float values flat.
/// We parse every parameter from python object and make from them Coord's.
/// Additionally it checks for correct length of coordinates parsed from Python values.
///
/// # Errors
///
/// May return error if cannot convert Python type into Rust one.
/// May return error if parsed number of coordinates is not expected by allowed length.
#[allow(clippy::needless_pass_by_value)]
pub fn build_geo_coords(
    py_parameters: Py<PyAny>,
    allowed_length_option: Option<usize>,
) -> RustPSQLDriverPyResult<Vec<Coord>> {
    let mut result_vec: Vec<Coord> = vec![];

    result_vec = Python::with_gil(|gil| {
        let bind_py_parameters = py_parameters.bind(gil);
        let parameters = py_sequence_to_rust(bind_py_parameters)?;

        let first_inner_bind_py_parameters = parameters[0].bind(gil);
        if first_inner_bind_py_parameters.is_instance_of::<PyFloat>()
            | first_inner_bind_py_parameters.is_instance_of::<PyInt>()
        {
            if parameters.len() % 2 != 0 {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "Length of coordinates that passed in flat structure must be a multiple of 2"
                        .into(),
                ));
            }

            for (pair_first_inner, pair_second_inner) in parameters.into_iter().tuples() {
                let coord_values =
                    convert_py_to_rust_coord_values(vec![pair_first_inner, pair_second_inner])?;
                result_vec.push(coord! {x: coord_values[0], y: coord_values[1]});
            }
        } else if first_inner_bind_py_parameters.is_instance_of::<PyList>()
            | first_inner_bind_py_parameters.is_instance_of::<PyTuple>()
            | first_inner_bind_py_parameters.is_instance_of::<PySet>()
        {
            for pair_inner_parameters in parameters {
                let bind_pair_inner_parameters = pair_inner_parameters.bind(gil);
                let pair_py_inner_parameters = py_sequence_to_rust(bind_pair_inner_parameters)?;

                if pair_py_inner_parameters.len() != 2 {
                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                        "Inner parameters must be pair(list/tuple/set) of int/float values".into(),
                    ));
                }

                let coord_values = convert_py_to_rust_coord_values(pair_py_inner_parameters)?;
                result_vec.push(coord! {x: coord_values[0], y: coord_values[1]});
            }
        } else {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                "Inner coordinates must be passed as pairs of int/float in list/tuple/set or as flat structure with int/float values".into(),
            ));
        };
        Ok::<Vec<Coord>, RustPSQLDriverError>(result_vec)
    })?;

    let number_of_coords = result_vec.len();
    let allowed_length = allowed_length_option.unwrap_or_default();

    if (allowed_length != 0) & (number_of_coords != allowed_length) {
        return Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
            "Invalid number of coordinates for this geo type, allowed {allowed_length}, got: {number_of_coords}"
        )));
    }

    Ok(result_vec)
}

/// Convert flat Python values with coordinates into vector of Geo values for building Geo types later.
///
/// Passed parameter can be either a list or a tuple or a set with elements.
/// We parse every parameter from python object and prepare them for making geo type.
/// Additionally it checks for correct length of coordinates parsed from Python values.
///
/// # Errors
///
/// May return error if cannot convert Python type into Rust one.
/// May return error if parsed number of coordinates is not expected by allowed length.
#[allow(clippy::needless_pass_by_value)]
pub fn build_flat_geo_coords(
    py_parameters: Py<PyAny>,
    allowed_length_option: Option<usize>,
) -> RustPSQLDriverPyResult<Vec<f64>> {
    Python::with_gil(|gil| {
        let allowed_length = allowed_length_option.unwrap_or_default();

        let bind_py_parameters = py_parameters.bind(gil);
        let parameters = py_sequence_to_rust(bind_py_parameters)?;
        let parameters_length = parameters.len();

        if (allowed_length != 0) & (parameters.len() != allowed_length) {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
                "Invalid number of values for this geo type, allowed {allowed_length}, got: {parameters_length}"
            )));
        };

        let result_vec = convert_py_to_rust_coord_values(parameters)?;

        let number_of_coords = result_vec.len();
        if (allowed_length != 0) & (number_of_coords != allowed_length) {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
                "Invalid number of values for this geo type, allowed {allowed_length}, got: {parameters_length}"
            )));
        };

        Ok::<Vec<f64>, RustPSQLDriverError>(result_vec)
    })
}
