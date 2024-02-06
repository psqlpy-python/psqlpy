use std::fmt::Debug;

use bytes::{BufMut, BytesMut};
use postgres_protocol::types;
use pyo3::{
    types::{PyBool, PyFloat, PyInt, PyList, PySet, PyString, PyTuple},
    Py, PyAny, Python, ToPyObject,
};
use tokio_postgres::{
    types::{to_sql_checked, ToSql, Type},
    Column, Row,
};

use crate::exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult};

#[derive(Debug, Clone, PartialEq)]
pub enum PythonType {
    PyNone,
    PyBool(bool),
    PyString(String),
    PyIntI32(i32),
    PyIntU32(u32),
    PyFloat32(f32),
    PyFloat64(f64),
    PyList(Vec<PythonType>),
    PyTuple(Vec<PythonType>),
}

impl PythonType {
    pub fn array_type(&self) -> RustPSQLDriverPyResult<tokio_postgres::types::Type> {
        match self {
            PythonType::PyString(_) => Ok(tokio_postgres::types::Type::TEXT_ARRAY),
            PythonType::PyIntI32(_) => Ok(tokio_postgres::types::Type::INT4_ARRAY),
            PythonType::PyIntU32(_) => Ok(tokio_postgres::types::Type::INT4_ARRAY),
            _ => Err(RustPSQLDriverError::PyToRustValueConversionError(
                "Can't process array type, your type doesn't have support yet".into(),
            )),
        }
    }
}

impl ToSql for PythonType {
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        if *self == PythonType::PyNone {
            return Ok(tokio_postgres::types::IsNull::Yes);
        }

        match self {
            PythonType::PyNone => {}
            PythonType::PyBool(boolean) => types::bool_to_sql(*boolean, out),
            // PythonType::PyString(string) => out.extend(string.as_bytes()),
            PythonType::PyString(string) => {
                <&str as ToSql>::to_sql(&string.as_str(), ty, out)?;
            }
            PythonType::PyIntI32(int) => out.put_i32(*int),
            PythonType::PyIntU32(int) => out.put_u32(*int),
            PythonType::PyFloat32(float) => out.put_f32(*float),
            PythonType::PyFloat64(float) => out.put_f64(*float),
            PythonType::PyList(py_iterable) | PythonType::PyTuple(py_iterable) => {
                let mut items = Vec::new();
                for inner in py_iterable.iter() {
                    items.push(inner);
                }
                // items.to_sql(&tokio_postgres::types::Type::TEXT_ARRAY, out)?;
                items.to_sql(&items[0].array_type()?, out)?;
            }
        }
        Ok(tokio_postgres::types::IsNull::No)
    }

    to_sql_checked!();
}

pub fn convert_parameters<'a>(parameters: &'a PyAny) -> RustPSQLDriverPyResult<Vec<PythonType>> {
    let mut result_vec: Vec<PythonType> = vec![];

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

pub fn py_to_rust(parameter: &PyAny) -> RustPSQLDriverPyResult<PythonType> {
    if parameter.is_none() {
        return Ok(PythonType::PyNone);
    } else if parameter.is_instance_of::<PyBool>() {
        return Ok(PythonType::PyBool(parameter.extract::<bool>()?));
    } else if parameter.is_instance_of::<PyString>() {
        return Ok(PythonType::PyString(parameter.extract::<String>()?));
    } else if parameter.is_instance_of::<PyFloat>() {
        // TODO: Add support for all types of float.
        return Ok(PythonType::PyFloat32(parameter.extract::<f32>()?));
    } else if parameter.is_instance_of::<PyInt>() {
        return Ok(PythonType::PyIntI32(parameter.extract::<i32>()?));
    } else if parameter.is_instance_of::<PyList>() | parameter.is_instance_of::<PyTuple>() {
        let mut items = Vec::new();
        for inner in parameter.iter()? {
            items.push(py_to_rust(inner?)?);
        }
        return Ok(PythonType::PyList(items));
    }
    Ok(PythonType::PyString("Can't convert!".to_string()))
}

pub fn postgres_to_py<'a>(
    py: Python<'a>,
    row: &Row,
    column: &Column,
    column_i: usize,
) -> RustPSQLDriverPyResult<Py<PyAny>> {
    match *column.type_() {
        // Convert TEXT and VARCHAR type into String, then into str
        Type::TEXT | Type::VARCHAR => Ok(row.try_get::<_, Option<String>>(column_i)?.to_object(py)),
        // Convert BOOL type into bool
        Type::BOOL => Ok(row.try_get::<_, Option<bool>>(column_i)?.to_object(py)),
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
        // Convert ARRAY of TEXT or VARCHAR into Vec<String>, then into list[str]
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => Ok(row
            .try_get::<_, Option<Vec<String>>>(column_i)?
            .to_object(py)),
        // Convert ARRAY of SmallInt into Vec<i16>, then into list[int]
        Type::INT2_ARRAY => Ok(row.try_get::<_, Option<Vec<i16>>>(column_i)?.to_object(py)),
        // Convert ARRAY of Integer into Vec<i32>, then into list[int]
        Type::INT4_ARRAY => Ok(row.try_get::<_, Option<Vec<i32>>>(column_i)?.to_object(py)),
        // Convert ARRAY of BigInt into Vec<i64>, then into list[int]
        Type::INT8_ARRAY => Ok(row.try_get::<_, Option<Vec<i64>>>(column_i)?.to_object(py)),
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
