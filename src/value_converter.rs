use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use macaddr::{MacAddr6, MacAddr8};
use postgres_types::{Field, FromSql, Kind, ToSql};
use rust_decimal::Decimal;
use serde_json::{json, Map, Value};
use std::{fmt::Debug, net::IpAddr};
use uuid::Uuid;

use bytes::{BufMut, BytesMut};
use postgres_protocol::types;
use pyo3::{
    sync::GILOnceCell,
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDate, PyDateTime, PyDict, PyDictMethods, PyFloat, PyInt,
        PyList, PyListMethods, PyString, PyTime, PyTuple, PyType, PyTypeMethods,
    },
    Bound, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};
use tokio_postgres::{
    types::{to_sql_checked, Type},
    Column, Row,
};

use crate::{
    additional_types::{RustMacAddr6, RustMacAddr8},
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    extra_types::{
        BigInt, Float32, Float64, Integer, Money, PyCustomType, PyJSON, PyJSONB, PyMacAddr6,
        PyMacAddr8, PyText, PyVarChar, SmallInt,
    },
};

static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

pub type QueryParameter = (dyn ToSql + Sync);

fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    DECIMAL_CLS
        .get_or_try_init(py, || {
            let type_object = py
                .import_bound("decimal")?
                .getattr("Decimal")?
                .downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
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

/// Additional type for types come from Python.
///
/// It's necessary because we need to pass this
/// enum into `to_sql` method of `ToSql` trait from
/// `postgres` crate.
#[derive(Debug, Clone, PartialEq)]
pub enum PythonDTO {
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
    PyIpAddress(IpAddr),
    PyList(Vec<PythonDTO>),
    PyTuple(Vec<PythonDTO>),
    PyJsonb(Value),
    PyJson(Value),
    PyMacAddr6(MacAddr6),
    PyMacAddr8(MacAddr8),
    PyDecimal(Decimal),
    PyCustomType(Vec<u8>),
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
            PythonDTO::PyIpAddress(pyidaddress) => {
                <&IpAddr as ToSql>::to_sql(&pyidaddress, ty, out)?;
            }
            PythonDTO::PyMacAddr6(pymacaddr) => {
                <&[u8] as ToSql>::to_sql(&pymacaddr.as_bytes(), ty, out)?;
            }
            PythonDTO::PyMacAddr8(pymacaddr) => {
                <&[u8] as ToSql>::to_sql(&pymacaddr.as_bytes(), ty, out)?;
            }
            PythonDTO::PyList(py_iterable) | PythonDTO::PyTuple(py_iterable) => {
                let mut items = Vec::new();
                for inner in py_iterable {
                    items.push(inner);
                }
                if items.len() > 1 {
                    items.to_sql(&items[0].array_type()?, out)?;
                } else {
                    return_is_null_true = true;
                }
            }
            PythonDTO::PyJsonb(py_dict) | PythonDTO::PyJson(py_dict) => {
                <&Value as ToSql>::to_sql(&py_dict, ty, out)?;
            }
            PythonDTO::PyDecimal(py_decimal) => {
                <Decimal as ToSql>::to_sql(py_decimal, ty, out)?;
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
pub fn convert_parameters(parameters: Py<PyAny>) -> RustPSQLDriverPyResult<Vec<PythonDTO>> {
    let mut result_vec: Vec<PythonDTO> = vec![];

    result_vec = Python::with_gil(|gil| {
        let params = parameters.extract::<Vec<Py<PyAny>>>(gil).map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError(
                "Cannot convert you parameters argument into Rust type, please use List/Tuple"
                    .into(),
            )
        })?;
        for parameter in params {
            result_vec.push(py_to_rust(parameter.bind(gil))?);
        }
        Ok::<Vec<PythonDTO>, RustPSQLDriverError>(result_vec)
    })?;
    Ok(result_vec)
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

    if parameter.is_instance_of::<PyCustomType>() {
        return Ok(PythonDTO::PyCustomType(
            parameter.extract::<PyCustomType>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<PyBool>() {
        return Ok(PythonDTO::PyBool(parameter.extract::<bool>()?));
    }

    if parameter.is_instance_of::<PyBytes>() {
        return Ok(PythonDTO::PyBytes(parameter.extract::<Vec<u8>>()?));
    }

    if parameter.is_instance_of::<PyText>() {
        return Ok(PythonDTO::PyText(parameter.extract::<PyText>()?.inner()));
    }

    if parameter.is_instance_of::<PyVarChar>() {
        return Ok(PythonDTO::PyVarChar(
            parameter.extract::<PyVarChar>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<PyString>() {
        return Ok(PythonDTO::PyString(parameter.extract::<String>()?));
    }

    if parameter.is_instance_of::<PyFloat>() {
        return Ok(PythonDTO::PyFloat32(parameter.extract::<f32>()?));
    }

    if parameter.is_instance_of::<Float32>() {
        return Ok(PythonDTO::PyFloat32(
            parameter.extract::<Float32>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<Float64>() {
        return Ok(PythonDTO::PyFloat64(
            parameter.extract::<Float64>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<SmallInt>() {
        return Ok(PythonDTO::PyIntI16(
            parameter.extract::<SmallInt>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<Integer>() {
        return Ok(PythonDTO::PyIntI32(
            parameter.extract::<Integer>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<BigInt>() {
        return Ok(PythonDTO::PyIntI64(
            parameter.extract::<BigInt>()?.retrieve_value(),
        ));
    }

    if parameter.is_instance_of::<Money>() {
        return Ok(PythonDTO::PyMoney(
            parameter.extract::<Money>()?.retrieve_value(),
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

    if parameter.is_instance_of::<PyList>() | parameter.is_instance_of::<PyTuple>() {
        let mut items = Vec::new();
        for inner in parameter.iter()? {
            items.push(py_to_rust(&inner?)?);
        }
        return Ok(PythonDTO::PyList(items));
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

    if parameter.is_instance_of::<PyJSONB>() {
        return Ok(PythonDTO::PyJsonb(
            parameter.extract::<PyJSONB>()?.inner().clone(),
        ));
    }

    if parameter.is_instance_of::<PyJSON>() {
        return Ok(PythonDTO::PyJson(
            parameter.extract::<PyJSON>()?.inner().clone(),
        ));
    }

    if parameter.is_instance_of::<PyMacAddr6>() {
        return Ok(PythonDTO::PyMacAddr6(
            parameter.extract::<PyMacAddr6>()?.inner(),
        ));
    }

    if parameter.is_instance_of::<PyMacAddr8>() {
        return Ok(PythonDTO::PyMacAddr8(
            parameter.extract::<PyMacAddr8>()?.inner(),
        ));
    }

    if parameter.get_type().name()? == "UUID" {
        return Ok(PythonDTO::PyUUID(Uuid::parse_str(
            parameter.str()?.extract::<&str>()?,
        )?));
    }

    if parameter.get_type().name()? == "decimal.Decimal" {
        return Ok(PythonDTO::PyDecimal(Decimal::from_str_exact(
            parameter.str()?.extract::<&str>()?,
        )?));
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

fn _composite_field_postgres_to_py<'a, T: FromSql<'a>>(
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
        Type::BYTEA => Ok(_composite_field_postgres_to_py::<Option<Vec<u8>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // // ---------- String Types ----------
        // // Convert TEXT and VARCHAR type into String, then into str
        Type::TEXT | Type::VARCHAR | Type::XML => Ok(_composite_field_postgres_to_py::<Option<String>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // ---------- Boolean Types ----------
        // Convert BOOL type into bool
        Type::BOOL => Ok(
            _composite_field_postgres_to_py::<Option<bool>>(type_, buf, is_simple)?.to_object(py),
        ),
        // ---------- Number Types ----------
        // Convert SmallInt into i16, then into int
        Type::INT2 => Ok(
            _composite_field_postgres_to_py::<Option<i16>>(type_, buf, is_simple)?.to_object(py),
        ),
        // Convert Integer into i32, then into int
        Type::INT4 => Ok(
            _composite_field_postgres_to_py::<Option<i32>>(type_, buf, is_simple)?.to_object(py),
        ),
        // Convert BigInt into i64, then into int
        Type::INT8 | Type::MONEY => Ok(
            _composite_field_postgres_to_py::<Option<i64>>(type_, buf, is_simple)?.to_object(py),
        ),
        // Convert REAL into f32, then into float
        Type::FLOAT4 => Ok(
            _composite_field_postgres_to_py::<Option<f32>>(type_, buf, is_simple)?.to_object(py),
        ),
        // Convert DOUBLE PRECISION into f64, then into float
        Type::FLOAT8 => Ok(
            _composite_field_postgres_to_py::<Option<f64>>(type_, buf, is_simple)?.to_object(py),
        ),
        // ---------- Date Types ----------
        // Convert DATE into NaiveDate, then into datetime.date
        Type::DATE => Ok(_composite_field_postgres_to_py::<Option<NaiveDate>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert Time into NaiveTime, then into datetime.time
        Type::TIME => Ok(_composite_field_postgres_to_py::<Option<NaiveTime>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMP => Ok(_composite_field_postgres_to_py::<Option<NaiveDateTime>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMPTZ => Ok(
            _composite_field_postgres_to_py::<Option<DateTime<FixedOffset>>>(
                type_, buf, is_simple,
            )?
            .to_object(py),
        ),
        // ---------- UUID Types ----------
        // Convert UUID into Uuid type, then into String if possible
        Type::UUID => {
            let rust_uuid = _composite_field_postgres_to_py::<Option<Uuid>>(type_, buf, is_simple)?;
            match rust_uuid {
                Some(rust_uuid) => {
                    return Ok(PyString::new_bound(py, &rust_uuid.to_string()).to_object(py))
                }
                None => Ok(py.None()),
            }
        }
        // ---------- IpAddress Types ----------
        Type::INET => Ok(
            _composite_field_postgres_to_py::<Option<IpAddr>>(type_, buf, is_simple)?.to_object(py),
        ),
        // Convert JSON/JSONB into Serde Value, then into list or dict
        Type::JSONB | Type::JSON => {
            let db_json = _composite_field_postgres_to_py::<Option<Value>>(type_, buf, is_simple)?;

            match db_json {
                Some(value) => Ok(build_python_from_serde_value(py, value)?),
                None => Ok(py.None().to_object(py)),
            }
        }
        // Convert MACADDR into inner type for macaddr6, then into str
        Type::MACADDR => {
            let macaddr_ =
                _composite_field_postgres_to_py::<Option<RustMacAddr6>>(type_, buf, is_simple)?;
            if let Some(macaddr_) = macaddr_ {
                Ok(macaddr_.inner().to_string().to_object(py))
            } else {
                Ok(py.None().to_object(py))
            }
        }
        Type::MACADDR8 => {
            let macaddr_ =
                _composite_field_postgres_to_py::<Option<RustMacAddr8>>(type_, buf, is_simple)?;
            if let Some(macaddr_) = macaddr_ {
                Ok(macaddr_.inner().to_string().to_object(py))
            } else {
                Ok(py.None().to_object(py))
            }
        }
        Type::NUMERIC => {
            if let Some(numeric_) = _composite_field_postgres_to_py::<Option<Decimal>>(
                type_, buf, is_simple,
            )? {
                return Ok(InnerDecimal(numeric_).to_object(py));
            }
            Ok(py.None().to_object(py))
        }
        // ---------- Array Text Types ----------
        Type::BOOL_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<bool>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of TEXT or VARCHAR into Vec<String>, then into list[str]
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::XML_ARRAY => Ok(_composite_field_postgres_to_py::<
            Option<Vec<String>>,
        >(type_, buf, is_simple)?
        .to_object(py)),
        // ---------- Array Integer Types ----------
        // Convert ARRAY of SmallInt into Vec<i16>, then into list[int]
        Type::INT2_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<i16>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of Integer into Vec<i32>, then into list[int]
        Type::INT4_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<i32>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of BigInt into Vec<i64>, then into list[int]
        Type::INT8_ARRAY | Type::MONEY_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<i64>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of Float4 into Vec<f32>, then into list[float]
        Type::FLOAT4_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<f32>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of Float8 into Vec<f64>, then into list[float]
        Type::FLOAT8_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<f64>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of Date into Vec<NaiveDate>, then into list[datetime.date]
        Type::DATE_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<NaiveDate>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of Time into Vec<NaiveTime>, then into list[datetime.date]
        Type::TIME_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<NaiveTime>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        // Convert ARRAY of TIMESTAMP into Vec<NaiveDateTime>, then into list[datetime.date]
        Type::TIMESTAMP_ARRAY => Ok(
            _composite_field_postgres_to_py::<Option<Vec<NaiveDateTime>>>(type_, buf, is_simple)?
                .to_object(py),
        ),
        // Convert ARRAY of TIMESTAMPTZ into Vec<DateTime<FixedOffset>>, then into list[datetime.date]
        Type::TIMESTAMPTZ_ARRAY => Ok(_composite_field_postgres_to_py::<
            Option<Vec<DateTime<FixedOffset>>>,
        >(type_, buf, is_simple)?
        .to_object(py)),
        // Convert ARRAY of UUID into Vec<DateTime<FixedOffset>>, then into list[datetime.date]
        Type::UUID_ARRAY => {
            let uuid_array =
                _composite_field_postgres_to_py::<Option<Vec<Uuid>>>(type_, buf, is_simple)?;
            match uuid_array {
                Some(rust_uuid_vec) => {
                    return Ok(PyList::new_bound(
                        py,
                        rust_uuid_vec
                            .iter()
                            .map(|rust_uuid| rust_uuid.to_string().as_str().to_object(py))
                            .collect::<Vec<Py<PyAny>>>(),
                    )
                    .to_object(py))
                }
                None => Ok(py.None().to_object(py)),
            }
        }
        // Convert ARRAY of INET into Vec<INET>, then into list[IPv4Address | IPv6Address]
        Type::INET_ARRAY => Ok(_composite_field_postgres_to_py::<Option<Vec<IpAddr>>>(
            type_, buf, is_simple,
        )?
        .to_object(py)),
        Type::JSONB_ARRAY | Type::JSON_ARRAY => {
            let db_json_array =
                _composite_field_postgres_to_py::<Option<Vec<Value>>>(type_, buf, is_simple)?;

            match db_json_array {
                Some(value) => {
                    let py_list = PyList::empty_bound(py);
                    for json_elem in value {
                        py_list.append(build_python_from_serde_value(py, json_elem)?)?;
                    }
                    Ok(py_list.to_object(py))
                }
                None => Ok(py.None().to_object(py)),
            }
        }
        Type::NUMERIC_ARRAY => {
            if let Some(numeric_array) = _composite_field_postgres_to_py::<Option<Vec<Decimal>>>(
                type_, buf, is_simple,
            )? {
                let py_list = PyList::empty_bound(py);
                for numeric_ in numeric_array {
                    py_list.append(InnerDecimal(numeric_).to_object(py))?;
                }
                return Ok(py_list.to_object(py))
            };
            Ok(py.None().to_object(py))
        },
        _ => Err(RustPSQLDriverError::RustToPyValueConversionError(
            format!("Cannot convert {type_} into Python type, please look at the custom_decoders functionality.")
        )),
    }
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
                        "PyJSON supports only list of lists or list of dicts.".to_string(),
                    ));
                }
            }
            Ok(json!(result_vec))
        } else if bind_value.is_instance_of::<PyDict>() {
            return py_to_rust(bind_value)?.to_serde_value();
        } else {
            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                "PyJSON must be list value.".to_string(),
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
