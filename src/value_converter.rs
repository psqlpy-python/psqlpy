use pyo3::{types::PyString, Py, PyAny, Python, ToPyObject};
use serde_json::Map;
use tokio_postgres::{
    types::{FromSql, Type},
    Column, Row,
};

use crate::engine::RustEnginePyResult;

pub fn postgres_to_py<'a>(
    py: Python<'a>,
    row: &Row,
    column: &Column,
    column_i: usize,
) -> RustEnginePyResult<Py<PyAny>> {
    match *column.type_() {
        Type::TEXT | Type::VARCHAR => {
            let rust_type = row.try_get::<_, Option<String>>(column_i).unwrap().unwrap();
            Ok(rust_type.to_object(py))
        }
        Type::BOOL => {
            let rust_type = row.try_get::<_, Option<bool>>(column_i).unwrap().unwrap();
            Ok(rust_type.to_object(py))
        }
        _ => Ok("can't convert".to_object(py)),
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
