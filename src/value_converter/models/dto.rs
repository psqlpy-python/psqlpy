use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use geo_types::{Line as LineSegment, LineString, Point, Rect};
use macaddr::{MacAddr6, MacAddr8};
use pg_interval::Interval;
use postgres_types::ToSql;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::{fmt::Debug, net::IpAddr};
use uuid::Uuid;

use bytes::{BufMut, BytesMut};
use postgres_protocol::types;
use pyo3::{PyObject, Python, ToPyObject};
use tokio_postgres::types::{to_sql_checked, Type};

use crate::{
    additional_types::{Circle, Line, RustLineSegment, RustLineString, RustPoint, RustRect},
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
};
use pgvector::Vector as PgVector;
use postgres_array::{array::Array, Dimension};

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
