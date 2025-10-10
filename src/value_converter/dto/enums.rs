use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use geo_types::{Line as LineSegment, LineString, Point, Rect};
use macaddr::{MacAddr6, MacAddr8};
use pg_interval::Interval;
use postgres_types::Type;
use rust_decimal::Decimal;
use serde_json::Value;
use std::{fmt::Debug, net::IpAddr};
use uuid::Uuid;

use crate::value_converter::additional_types::{Circle, Line};
use postgres_array::array::Array;

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
    PyList(Vec<PythonDTO>, Type),
    PyArray(Array<PythonDTO>, Type),
    PyTuple(Vec<PythonDTO>, Type),
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
