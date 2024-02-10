use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use std::fmt::Debug;
use uuid::Uuid;

use bytes::{BufMut, BytesMut};
use postgres_protocol::types;
use pyo3::{
    types::{
        PyBool, PyBytes, PyDate, PyDateTime, PyFloat, PyInt, PyList, PySet, PyString, PyTime,
        PyTuple,
    },
    Py, PyAny, Python, ToPyObject,
};
use tokio_postgres::{
    types::{to_sql_checked, ToSql, Type},
    Column, Row,
};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    extra_types::{BigInt, Integer, PyUUID, SmallInt},
};

/// Additional type for types come from Python.
///
/// It's necessary because we need to pass this
/// enum into `to_sql` method of ToSql trait from
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
    PyFloat32(f32),
    PyFloat64(f64),
    PyDate(NaiveDate),
    PyTime(NaiveTime),
    PyDateTime(NaiveDateTime),
    PyDateTimeTz(DateTime<FixedOffset>),
    PyList(Vec<PythonDTO>),
    PyTuple(Vec<PythonDTO>),
}

/// Implement necessary methods for this type.
impl PythonDTO {
    /// Return type of the Array for PostgreSQL.
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
            PythonDTO::PyIntI32(_) => Ok(tokio_postgres::types::Type::INT4_ARRAY),
            PythonDTO::PyIntI64(_) => Ok(tokio_postgres::types::Type::INT8_ARRAY),
            PythonDTO::PyFloat32(_) => Ok(tokio_postgres::types::Type::FLOAT4_ARRAY),
            PythonDTO::PyFloat64(_) => Ok(tokio_postgres::types::Type::FLOAT8_ARRAY),
            PythonDTO::PyIntU32(_) => Ok(tokio_postgres::types::Type::INT4_ARRAY),
            _ => Err(RustPSQLDriverError::PyToRustValueConversionError(
                "Can't process array type, your type doesn't have support yet".into(),
            )),
        }
    }
}

/// Implement ToSql trait.
///
/// It allows us to pass PythonDTO enum as parameter
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
    /// We convert every inner type of PythonDTO enum variant
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
        if *self == PythonDTO::PyNone {
            return Ok(tokio_postgres::types::IsNull::Yes);
        }

        match self {
            PythonDTO::PyNone => {}
            PythonDTO::PyBytes(pybytes) => {
                <Vec<u8> as ToSql>::to_sql(pybytes, ty, out)?;
            }
            PythonDTO::PyBool(boolean) => types::bool_to_sql(*boolean, out),
            PythonDTO::PyUUID(pyuuid) => {
                <Uuid as ToSql>::to_sql(&pyuuid, ty, out)?;
            }
            PythonDTO::PyString(string) => {
                <&str as ToSql>::to_sql(&string.as_str(), ty, out)?;
            }
            PythonDTO::PyIntI16(int) => out.put_i16(*int),
            PythonDTO::PyIntI32(int) => out.put_i32(*int),
            PythonDTO::PyIntI64(int) => out.put_i64(*int),
            PythonDTO::PyIntU32(int) => out.put_u32(*int),
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
            PythonDTO::PyList(py_iterable) | PythonDTO::PyTuple(py_iterable) => {
                let mut items = Vec::new();
                for inner in py_iterable.iter() {
                    items.push(inner);
                }
                items.to_sql(&items[0].array_type()?, out)?;
            }
        }
        Ok(tokio_postgres::types::IsNull::No)
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
/// # Errors:
///
/// May return Err Result if can't convert python object.
pub fn convert_parameters<'a>(parameters: &'a PyAny) -> RustPSQLDriverPyResult<Vec<PythonDTO>> {
    let mut result_vec: Vec<PythonDTO> = vec![];

    if parameters.is_instance_of::<PyList>()
        || parameters.is_instance_of::<PyTuple>()
        || parameters.is_instance_of::<PySet>()
    {
        let params = parameters.extract::<Vec<&PyAny>>()?;
        for parameter in params.iter() {
            result_vec.push(py_to_rust(parameter)?);
        }
    }
    return Ok(result_vec);
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

        match timestamp_tz {
            Ok(pydatetime_tz) => return Ok(PythonDTO::PyDateTimeTz(pydatetime_tz)),
            Err(_) => {}
        }

        let timestamp_no_tz = parameter.extract::<NaiveDateTime>();

        match timestamp_no_tz {
            Ok(pydatetime_no_tz) => return Ok(PythonDTO::PyDateTime(pydatetime_no_tz)),
            Err(_) => {}
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

    Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
        "Can not covert you type {} into inner one",
        parameter,
    )))
}

/// Convert type from postgres to python type.
///
/// # Errors:
///
/// May return Err Result if cannot convert postgres
/// type into rust one.
pub fn postgres_to_py<'a>(
    py: Python<'a>,
    row: &Row,
    column: &Column,
    column_i: usize,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    match *column.type_() {
        // ---------- Bytes Types ----------
        // Convert BYTEA type into Vector<u8>, then into PyBytes
        Type::BYTEA => match row.try_get::<_, Option<Vec<u8>>>(column_i)? {
            Some(rest_bytes) => Ok(PyBytes::new(py, &rest_bytes).to_object(py)),
            None => return Ok(py.None()),
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
                None => return Ok(py.None()),
            }
        }
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
            None => return Ok(py.None().to_object(py)),
        },
        _ => Err(RustPSQLDriverError::RustToPyValueConversionError(
            column.type_().to_string(),
        )),
    }
}

// pub fn postgres_row_to_json_value(row: &Row) -> Result<String, Error> {
//     let row_data = postgres_row_to_row_data(row)?;
//     let a = JSONValue::Object(row_data);
//     Ok(a.to_string())
// }

// // some type-aliases I use in my project
// pub type JSONValue = serde_json::Value;
// pub type RowData = Map<String, JSONValue>;
// pub type Error = anyhow::Error; // from: https://github.com/dtolnay/anyhow

// pub fn postgres_row_to_row_data(row: &Row) -> Result<RowData, Error> {
//     let mut result: Map<String, JSONValue> = Map::new();
//     for (i, column) in row.columns().iter().enumerate() {
//         let name = column.name();
//         let json_value = pg_cell_to_json_value(&row, column, i)?;
//         result.insert(name.to_string(), json_value);
//     }
//     Ok(result)
// }

// pub fn pg_cell_to_json_value(
//     row: &Row,
//     column: &Column,
//     column_i: usize,
// ) -> Result<JSONValue, Error> {
//     // let f64_to_json_number = |raw_val: f64| -> Result<JSONValue, Error> {
//     //     let temp = serde_json::Number::from_f64(raw_val.into())
//     //         .ok_or(anyhow::anyhow!("invalid json-float"))?;
//     //     Ok(JSONValue::Number(temp))
//     // };
//     Ok(match *column.type_() {
//         // single types
//         Type::BOOL => get_basic(row, column, column_i, |a: bool| Ok(JSONValue::Bool(a)))?,
//         Type::INT2 => get_basic(row, column, column_i, |a: i16| {
//             Ok(JSONValue::Number(serde_json::Number::from(a)))
//         })?,
//         Type::INT4 => get_basic(row, column, column_i, |a: i32| {
//             Ok(JSONValue::Number(serde_json::Number::from(a)))
//         })?,
//         Type::INT8 => get_basic(row, column, column_i, |a: i64| {
//             Ok(JSONValue::Number(serde_json::Number::from(a)))
//         })?,
//         Type::TEXT | Type::VARCHAR => {
//             get_basic(row, column, column_i, |a: String| Ok(JSONValue::String(a)))?
//         }
//         Type::JSON | Type::JSONB => get_basic(row, column, column_i, |a: JSONValue| Ok(a))?,
//         // Type::FLOAT4 => get_basic(row, column, column_i, |a: f32| {
//         //     Ok(f64_to_json_number(a.into())?)
//         // })?,
//         // Type::FLOAT8 => get_basic(row, column, column_i, |a: f64| Ok(f64_to_json_number(a)?))?,
//         // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
//         Type::TS_VECTOR => get_basic(row, column, column_i, |a: StringCollector| {
//             Ok(JSONValue::String(a.0))
//         })?,

//         // array types
//         Type::BOOL_ARRAY => get_array(row, column, column_i, |a: bool| Ok(JSONValue::Bool(a)))?,
//         Type::INT2_ARRAY => get_array(row, column, column_i, |a: i16| {
//             Ok(JSONValue::Number(serde_json::Number::from(a)))
//         })?,
//         Type::INT4_ARRAY => get_array(row, column, column_i, |a: i32| {
//             Ok(JSONValue::Number(serde_json::Number::from(a)))
//         })?,
//         Type::INT8_ARRAY => get_array(row, column, column_i, |a: i64| {
//             Ok(JSONValue::Number(serde_json::Number::from(a)))
//         })?,
//         Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => {
//             get_array(row, column, column_i, |a: String| Ok(JSONValue::String(a)))?
//         }
//         Type::JSON_ARRAY | Type::JSONB_ARRAY => {
//             get_array(row, column, column_i, |a: JSONValue| Ok(a))?
//         }
//         // Type::FLOAT4_ARRAY => get_array(row, column, column_i, |a: f32| {
//         //     Ok(f64_to_json_number(a.into())?)
//         // })?,
//         // Type::FLOAT8_ARRAY => {
//         //     get_array(row, column, column_i, |a: f64| Ok(f64_to_json_number(a)?))?
//         // }
//         // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
//         Type::TS_VECTOR_ARRAY => get_array(row, column, column_i, |a: StringCollector| {
//             Ok(JSONValue::String(a.0))
//         })?,

//         _ => anyhow::bail!(
//             "Cannot convert pg-cell \"{}\" of type \"{}\" to a JSONValue.",
//             column.name(),
//             column.type_().name()
//         ),
//     })
// }

// fn get_basic<'a, T: FromSql<'a>>(
//     row: &'a Row,
//     column: &Column,
//     column_i: usize,
//     val_to_json_val: impl Fn(T) -> Result<JSONValue, Error>,
// ) -> T {
//     let rust_type = row.try_get::<_, Option<T>>(column_i).unwrap().unwrap();
// }
// fn get_array<'a, T: FromSql<'a>>(
//     row: &'a Row,
//     column: &Column,
//     column_i: usize,
//     val_to_json_val: impl Fn(T) -> Result<JSONValue, Error>,
// ) -> Result<JSONValue, Error> {
//     let raw_val_array = row.try_get::<_, Option<Vec<T>>>(column_i)?;
//     Ok(match raw_val_array {
//         Some(val_array) => {
//             let mut result = vec![];
//             for val in val_array {
//                 result.push(val_to_json_val(val)?);
//             }
//             JSONValue::Array(result)
//         }
//         None => JSONValue::Null,
//     })
// }

// // you can remove this section if not using TS_VECTOR (or other types requiring an intermediary `FromSQL` struct)
// struct StringCollector(String);
// impl FromSql<'_> for StringCollector {
//     fn from_sql(
//         _: &Type,
//         raw: &[u8],
//     ) -> Result<StringCollector, Box<dyn std::error::Error + Sync + Send>> {
//         let result = std::str::from_utf8(raw)?;
//         Ok(StringCollector(result.to_owned()))
//     }
//     fn accepts(_ty: &Type) -> bool {
//         true
//     }
// }
