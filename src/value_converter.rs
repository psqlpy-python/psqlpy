use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::{json, Map, Value};
use std::{fmt::Debug, net::IpAddr};
use uuid::Uuid;

use bytes::{BufMut, BytesMut};
use postgres_protocol::types;
use pyo3::{
    types::{
        PyBool, PyBytes, PyDate, PyDateTime, PyDict, PyFloat, PyInt, PyList, PySet, PyString,
        PyTime, PyTuple,
    },
    Py, PyAny, Python, ToPyObject,
};
use tokio_postgres::{
    types::{to_sql_checked, ToSql, Type},
    Column, Row,
};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    extra_types::{BigInt, Integer, PyJSON, PyUUID, SmallInt},
};

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
    PyString(String),
    PyIntI16(i16),
    PyIntI32(i32),
    PyIntI64(i64),
    PyIntU32(u32),
    PyIntU64(u64),
    PyFloat32(f32),
    PyFloat64(f64),
    PyDate(NaiveDate),
    PyTime(NaiveTime),
    PyDateTime(NaiveDateTime),
    PyDateTimeTz(DateTime<FixedOffset>),
    PyIpAddress(IpAddr),
    PyList(Vec<PythonDTO>),
    PyTuple(Vec<PythonDTO>),
    PyJson(Value),
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
            PythonDTO::PyUUID(_) => Ok(tokio_postgres::types::Type::UUID_ARRAY),
            PythonDTO::PyString(_) => Ok(tokio_postgres::types::Type::TEXT_ARRAY),
            PythonDTO::PyIntI16(_) => Ok(tokio_postgres::types::Type::INT2_ARRAY),
            PythonDTO::PyIntI32(_) | PythonDTO::PyIntU32(_) => {
                Ok(tokio_postgres::types::Type::INT4_ARRAY)
            }
            PythonDTO::PyIntI64(_) => Ok(tokio_postgres::types::Type::INT8_ARRAY),
            PythonDTO::PyFloat32(_) => Ok(tokio_postgres::types::Type::FLOAT4_ARRAY),
            PythonDTO::PyFloat64(_) => Ok(tokio_postgres::types::Type::FLOAT8_ARRAY),
            PythonDTO::PyIpAddress(_) => Ok(tokio_postgres::types::Type::INET_ARRAY),
            PythonDTO::PyJson(_) => Ok(tokio_postgres::types::Type::JSONB_ARRAY),
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
            PythonDTO::PyString(pystring) => Ok(json!(pystring)),
            PythonDTO::PyIntI32(pyint) => Ok(json!(pyint)),
            PythonDTO::PyIntI64(pyint) => Ok(json!(pyint)),
            PythonDTO::PyIntU64(pyint) => Ok(json!(pyint)),
            PythonDTO::PyFloat64(pyfloat) => Ok(json!(pyfloat)),
            PythonDTO::PyList(pylist) => {
                let mut vec_serde_values: Vec<Value> = vec![];

                for py_object in pylist {
                    vec_serde_values.push(py_object.to_serde_value()?);
                }

                Ok(json!(vec_serde_values))
            }
            PythonDTO::PyJson(py_dict) => Ok(py_dict.clone()),
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
            PythonDTO::PyBytes(pybytes) => {
                <Vec<u8> as ToSql>::to_sql(pybytes, ty, out)?;
            }
            PythonDTO::PyBool(boolean) => types::bool_to_sql(*boolean, out),
            PythonDTO::PyUUID(pyuuid) => {
                <Uuid as ToSql>::to_sql(pyuuid, ty, out)?;
            }
            PythonDTO::PyString(string) => {
                <&str as ToSql>::to_sql(&string.as_str(), ty, out)?;
            }
            PythonDTO::PyIntI16(int) => out.put_i16(*int),
            PythonDTO::PyIntI32(int) => out.put_i32(*int),
            PythonDTO::PyIntI64(int) => out.put_i64(*int),
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
            PythonDTO::PyJson(py_dict) => {
                <&Value as ToSql>::to_sql(&py_dict, ty, out)?;
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
pub fn convert_parameters(parameters: &PyAny) -> RustPSQLDriverPyResult<Vec<PythonDTO>> {
    let mut result_vec: Vec<PythonDTO> = vec![];

    if parameters.is_instance_of::<PyList>()
        || parameters.is_instance_of::<PyTuple>()
        || parameters.is_instance_of::<PySet>()
    {
        let params = parameters.extract::<Vec<&PyAny>>()?;
        for parameter in params {
            result_vec.push(py_to_rust(parameter)?);
        }
    }
    Ok(result_vec)
}

/// Convert single python parameter to `PythonDTO` enum.
///
/// # Errors
///
/// May return Err Result if python type doesn't have support yet
/// or value of the type is incorrect.
pub fn py_to_rust(parameter: &PyAny) -> RustPSQLDriverPyResult<PythonDTO> {
    if parameter.is_none() {
        return Ok(PythonDTO::PyNone);
    }

    if parameter.is_instance_of::<PyBool>() {
        return Ok(PythonDTO::PyBool(parameter.extract::<bool>()?));
    }

    if parameter.is_instance_of::<PyBytes>() {
        return Ok(PythonDTO::PyBytes(parameter.extract::<Vec<u8>>()?));
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

    if parameter.is_instance_of::<PyUUID>() {
        return Ok(PythonDTO::PyUUID(parameter.extract::<PyUUID>()?.inner()));
    }

    if parameter.is_instance_of::<PyString>() {
        return Ok(PythonDTO::PyString(parameter.extract::<String>()?));
    }

    if parameter.is_instance_of::<PyFloat>() {
        // TODO: Add support for all types of float.
        return Ok(PythonDTO::PyFloat32(parameter.extract::<f32>()?));
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

    if parameter.is_instance_of::<PyInt>() {
        return Ok(PythonDTO::PyIntI32(parameter.extract::<i32>()?));
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
            items.push(py_to_rust(inner?)?);
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
            let value = py_to_rust(py_list.get_item(1)?)?;

            serde_map.insert(key, value.to_serde_value()?);
        }

        return Ok(PythonDTO::PyJson(Value::Object(serde_map)));
    }

    if parameter.is_instance_of::<PyJSON>() {
        return Ok(PythonDTO::PyJson(
            parameter.extract::<PyJSON>()?.inner().clone(),
        ));
    }

    if let Ok(id_address) = parameter.extract::<IpAddr>() {
        return Ok(PythonDTO::PyIpAddress(id_address));
    }

    Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
        "Can not covert you type {parameter} into inner one",
    )))
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
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    match *column.type_() {
        // ---------- Bytes Types ----------
        // Convert BYTEA type into Vector<u8>, then into PyBytes
        Type::BYTEA => match row.try_get::<_, Option<Vec<u8>>>(column_i)? {
            Some(rest_bytes) => Ok(PyBytes::new(py, &rest_bytes).to_object(py)),
            None => Ok(py.None()),
        },
        // ---------- String Types ----------
        // Convert TEXT and VARCHAR type into String, then into str
        Type::TEXT | Type::VARCHAR => Ok(row.try_get::<_, Option<String>>(column_i)?.to_object(py)),
        // ---------- Boolean Types ----------
        // Convert BOOL type into bool
        Type::BOOL => Ok(row.try_get::<_, Option<bool>>(column_i)?.to_object(py)),
        // ---------- Number Types ----------
        // Convert SmallInt into i16, then into int
        Type::INT2 => Ok(row.try_get::<_, Option<i16>>(column_i)?.to_object(py)),
        // Convert Integer into i32, then into int
        Type::INT4 => Ok(row.try_get::<_, Option<i32>>(column_i)?.to_object(py)),
        // Convert BigInt into i64, then into int
        Type::INT8 => Ok(row.try_get::<_, Option<i64>>(column_i)?.to_object(py)),
        // Convert REAL into f32, then into float
        Type::FLOAT4 => Ok(row.try_get::<_, Option<f32>>(column_i)?.to_object(py)),
        // Convert DOUBLE PRECISION into f64, then into float
        Type::FLOAT8 => Ok(row.try_get::<_, Option<f64>>(column_i)?.to_object(py)),
        // ---------- Date Types ----------
        // Convert DATE into NaiveDate, then into datetime.date
        Type::DATE => Ok(row.try_get::<_, Option<NaiveDate>>(column_i)?.to_object(py)),
        // Convert Time into NaiveTime, then into datetime.time
        Type::TIME => Ok(row.try_get::<_, Option<NaiveTime>>(column_i)?.to_object(py)),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMP => Ok(row
            .try_get::<_, Option<NaiveDateTime>>(column_i)?
            .to_object(py)),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMPTZ => Ok(row
            .try_get::<_, Option<DateTime<FixedOffset>>>(column_i)?
            .to_object(py)),
        // ---------- UUID Types ----------
        // Convert UUID into Uuid type, then into String if possible
        Type::UUID => {
            let rust_uuid = row.try_get::<_, Option<Uuid>>(column_i)?;
            match rust_uuid {
                Some(rust_uuid) => {
                    return Ok(PyString::new(py, &rust_uuid.to_string()).to_object(py))
                }
                None => Ok(py.None()),
            }
        }
        // ---------- IpAddress Types ----------
        Type::INET => Ok(row.try_get::<_, Option<IpAddr>>(column_i)?.to_object(py)),
        // ---------- Array Text Types ----------
        // Convert ARRAY of TEXT or VARCHAR into Vec<String>, then into list[str]
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => Ok(row
            .try_get::<_, Option<Vec<String>>>(column_i)?
            .to_object(py)),
        // ---------- Array Integer Types ----------
        // Convert ARRAY of SmallInt into Vec<i16>, then into list[int]
        Type::INT2_ARRAY => Ok(row.try_get::<_, Option<Vec<i16>>>(column_i)?.to_object(py)),
        // Convert ARRAY of Integer into Vec<i32>, then into list[int]
        Type::INT4_ARRAY => Ok(row.try_get::<_, Option<Vec<i32>>>(column_i)?.to_object(py)),
        // Convert ARRAY of BigInt into Vec<i64>, then into list[int]
        Type::INT8_ARRAY => Ok(row.try_get::<_, Option<Vec<i64>>>(column_i)?.to_object(py)),
        // Convert ARRAY of Float4 into Vec<f32>, then into list[float]
        Type::FLOAT4_ARRAY => Ok(row.try_get::<_, Option<Vec<f32>>>(column_i)?.to_object(py)),
        // Convert ARRAY of Float8 into Vec<f64>, then into list[float]
        Type::FLOAT8_ARRAY => Ok(row.try_get::<_, Option<Vec<f64>>>(column_i)?.to_object(py)),
        // Convert ARRAY of Date into Vec<NaiveDate>, then into list[datetime.date]
        Type::DATE_ARRAY => Ok(row
            .try_get::<_, Option<Vec<NaiveDate>>>(column_i)?
            .to_object(py)),
        // Convert ARRAY of Time into Vec<NaiveTime>, then into list[datetime.date]
        Type::TIME_ARRAY => Ok(row
            .try_get::<_, Option<Vec<NaiveTime>>>(column_i)?
            .to_object(py)),
        // Convert ARRAY of TIMESTAMP into Vec<NaiveDateTime>, then into list[datetime.date]
        Type::TIMESTAMP_ARRAY => Ok(row
            .try_get::<_, Option<Vec<NaiveDateTime>>>(column_i)?
            .to_object(py)),
        // Convert ARRAY of TIMESTAMPTZ into Vec<DateTime<FixedOffset>>, then into list[datetime.date]
        Type::TIMESTAMPTZ_ARRAY => Ok(row
            .try_get::<_, Option<Vec<DateTime<FixedOffset>>>>(column_i)?
            .to_object(py)),
        // Convert ARRAY of UUID into Vec<DateTime<FixedOffset>>, then into list[datetime.date]
        Type::UUID_ARRAY => match row.try_get::<_, Option<Vec<Uuid>>>(column_i)? {
            Some(rust_uuid_vec) => {
                return Ok(PyList::new(
                    py,
                    rust_uuid_vec
                        .iter()
                        .map(|rust_uuid| rust_uuid.to_string().as_str().to_object(py))
                        .collect::<Vec<Py<PyAny>>>(),
                )
                .to_object(py))
            }
            None => Ok(py.None().to_object(py)),
        },
        // Convert ARRAY of INET into Vec<INET>, then into list[IPv4Address | IPv6Address]
        Type::INET_ARRAY => Ok(row
            .try_get::<_, Option<Vec<IpAddr>>>(column_i)?
            .to_object(py)),
        Type::JSONB | Type::JSON => {
            let db_json = row.try_get::<_, Option<Value>>(column_i)?;

            match db_json {
                Some(value) => Ok(value.to_string().to_object(py)),
                None => Ok(py.None().to_object(py)),
            }
        }
        _ => Err(RustPSQLDriverError::RustToPyValueConversionError(
            column.type_().to_string(),
        )),
    }
}

/// Convert python List of Dict type or just Dict in serde `Value`.
///
/// # Errors
/// May return error if cannot convert Python type into Rust one.
pub fn build_serde_value(value: &PyAny) -> RustPSQLDriverPyResult<Value> {
    if value.is_instance_of::<PyList>() {
        let mut result_vec: Vec<Value> = vec![];

        let params: Vec<&PyAny> = value.extract::<Vec<&PyAny>>()?;

        for inner in &params {
            if inner.is_instance_of::<PyDict>() {
                let python_dto = py_to_rust(inner)?;
                result_vec.push(python_dto.to_serde_value()?);
            } else if inner.is_instance_of::<PyList>() {
                let serde_value = build_serde_value(inner)?;
                result_vec.push(serde_value);
            } else {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "PyJSON/PyJSONB supports only list of lists or list of dicts.".to_string(),
                ));
            }
        }
        Ok(json!(result_vec))
    } else if value.is_instance_of::<PyDict>() {
        return py_to_rust(value)?.to_serde_value();
    } else {
        return Err(RustPSQLDriverError::PyToRustValueConversionError(
            "PyJSON must be list value.".to_string(),
        ));
    }
}
