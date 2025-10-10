use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use pg_interval::Interval;
use postgres_array::{Array, Dimension};
use postgres_types::{Field, FromSql, Kind, Type};
use rust_decimal::Decimal;
use serde_json::Value;
use std::net::IpAddr;
use tokio_postgres::{Column, Row};
use uuid::Uuid;

use pyo3::{
    types::{PyAnyMethods, PyBytes, PyDict, PyDictMethods, PyList, PyListMethods, PyString},
    Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, Python,
};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    value_converter::{
        additional_types::{
            Circle, Line, RustLineSegment, RustLineString, RustMacAddr6, RustMacAddr8, RustPoint,
            RustRect,
        },
        models::{
            decimal::InnerDecimal, interval::InnerInterval, serde_value::InternalSerdeValue,
            uuid::InternalUuid,
        },
    },
};
use pgvector::Vector as PgVector;

/// Convert serde `Value` into Python object.
/// # Errors
/// May return Err Result if cannot add new value to Python Dict.
pub fn build_python_from_serde_value(py: Python<'_>, value: Value) -> PSQLPyResult<Py<PyAny>> {
    match value {
        Value::Array(massive) => {
            let mut result_vec: Vec<Py<PyAny>> = vec![];

            for single_record in massive {
                result_vec.push(build_python_from_serde_value(py, single_record)?);
            }

            Ok(result_vec.into_py_any(py)?)
        }
        Value::Object(mapping) => {
            let py_dict = PyDict::new(py);

            for (key, value) in mapping {
                py_dict.set_item(
                    build_python_from_serde_value(py, Value::String(key))?,
                    build_python_from_serde_value(py, value)?,
                )?;
            }
            Ok(py_dict.into_py_any(py)?)
        }
        Value::Bool(boolean) => Ok(boolean.into_py_any(py)?),
        Value::Number(number) => {
            if number.is_f64() {
                Ok(number.as_f64().into_py_any(py)?)
            } else if number.is_i64() {
                Ok(number.as_i64().into_py_any(py)?)
            } else {
                Ok(number.as_u64().into_py_any(py)?)
            }
        }
        Value::String(string) => Ok(string.into_py_any(py)?),
        Value::Null => Ok(py.None()),
    }
}

fn composite_field_postgres_to_py<'a, T: FromSql<'a>>(
    type_: &Type,
    buf: &mut &'a [u8],
    is_simple: bool,
) -> PSQLPyResult<T> {
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

/// Convert rust array to python list.
///
/// It can convert multidimensional arrays.
fn postgres_array_to_py<'py, T: IntoPyObject<'py> + Clone>(
    py: Python<'py>,
    array: Option<Array<T>>,
) -> Option<Py<PyList>> {
    array.map(|array| {
        // Collect data once instead of creating copies in recursion
        let data: Vec<T> = array.iter().cloned().collect();
        inner_postgres_array_to_py(py, array.dimensions(), &data, 0, 0)
    })
}

/// Inner postgres array conversion to python list.
#[allow(clippy::cast_sign_loss)]
fn inner_postgres_array_to_py<'py, T>(
    py: Python<'py>,
    dimensions: &[Dimension],
    data: &[T],
    dimension_index: usize,
    data_offset: usize,
) -> Py<PyList>
where
    T: IntoPyObject<'py> + Clone,
{
    // Check bounds early
    if dimension_index >= dimensions.len() || data_offset >= data.len() {
        return PyList::empty(py).unbind();
    }

    let current_dimension = &dimensions[dimension_index];
    let current_len = current_dimension.len as usize;

    // If this is the last dimension, create a list with the actual data
    if dimension_index + 1 >= dimensions.len() {
        let end_offset = (data_offset + current_len).min(data.len());
        let slice = &data[data_offset..end_offset];

        // Create Python list more efficiently
        return match PyList::new(py, slice.iter().cloned()) {
            Ok(list) => list.unbind(),
            Err(_) => PyList::empty(py).unbind(),
        };
    }

    // For multi-dimensional arrays, recursively create nested lists
    let final_list = PyList::empty(py);

    // Calculate the size of each sub-array
    let sub_array_size = dimensions[dimension_index + 1..]
        .iter()
        .map(|d| d.len as usize)
        .product::<usize>();

    let mut current_offset = data_offset;

    for _ in 0..current_len {
        if current_offset >= data.len() {
            break;
        }

        let inner_list =
            inner_postgres_array_to_py(py, dimensions, data, dimension_index + 1, current_offset);

        if final_list.append(inner_list).is_err() {
            break;
        }

        current_offset += sub_array_size;
    }

    final_list.unbind()
}

#[allow(clippy::too_many_lines)]
fn postgres_bytes_to_py(
    py: Python<'_>,
    type_: &Type,
    buf: &mut &[u8],
    is_simple: bool,
) -> PSQLPyResult<Py<PyAny>> {
    match *type_ {
        // ---------- Bytes Types ----------
        // Convert BYTEA type into Vector<u8>, then into PyBytes
        Type::BYTEA => {
            let vec_of_bytes =
                composite_field_postgres_to_py::<Option<Vec<u8>>>(type_, buf, is_simple)?;
            if let Some(vec_of_bytes) = vec_of_bytes {
                return Ok(PyBytes::new(py, &vec_of_bytes).into_py_any(py)?);
            }
            Ok(py.None())
        }
        Type::OID => Ok(
            composite_field_postgres_to_py::<Option<i32>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        Type::NAME => Ok(
            composite_field_postgres_to_py::<Option<String>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // // ---------- String Types ----------
        // // Convert TEXT and VARCHAR type into String, then into str
        Type::TEXT | Type::VARCHAR | Type::XML => Ok(composite_field_postgres_to_py::<
            Option<String>,
        >(type_, buf, is_simple)?
        .into_py_any(py)?),
        // ---------- Boolean Types ----------
        // Convert BOOL type into bool
        Type::BOOL => Ok(
            composite_field_postgres_to_py::<Option<bool>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // ---------- Number Types ----------
        // Convert SmallInt into i16, then into int
        Type::INT2 => Ok(
            composite_field_postgres_to_py::<Option<i16>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // Convert Integer into i32, then into int
        Type::INT4 => Ok(
            composite_field_postgres_to_py::<Option<i32>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // Convert BigInt into i64, then into int
        Type::INT8 | Type::MONEY => Ok(composite_field_postgres_to_py::<Option<i64>>(
            type_, buf, is_simple,
        )?
        .into_py_any(py)?),
        // Convert REAL into f32, then into float
        Type::FLOAT4 => Ok(
            composite_field_postgres_to_py::<Option<f32>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // Convert DOUBLE PRECISION into f64, then into float
        Type::FLOAT8 => Ok(
            composite_field_postgres_to_py::<Option<f64>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // ---------- Date Types ----------
        // Convert DATE into NaiveDate, then into datetime.date
        Type::DATE => Ok(composite_field_postgres_to_py::<Option<NaiveDate>>(
            type_, buf, is_simple,
        )?
        .into_py_any(py)?),
        // Convert Time into NaiveTime, then into datetime.time
        Type::TIME => Ok(composite_field_postgres_to_py::<Option<NaiveTime>>(
            type_, buf, is_simple,
        )?
        .into_py_any(py)?),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMP => Ok(composite_field_postgres_to_py::<Option<NaiveDateTime>>(
            type_, buf, is_simple,
        )?
        .into_py_any(py)?),
        // Convert TIMESTAMP into NaiveDateTime, then into datetime.datetime
        Type::TIMESTAMPTZ => Ok(
            composite_field_postgres_to_py::<Option<DateTime<FixedOffset>>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // ---------- UUID Types ----------
        // Convert UUID into Uuid type, then into String if possible
        Type::UUID => {
            let rust_uuid = composite_field_postgres_to_py::<Option<Uuid>>(type_, buf, is_simple)?;
            match rust_uuid {
                Some(rust_uuid) => Ok(PyString::new(py, &rust_uuid.to_string()).into_py_any(py)?),
                None => Ok(py.None()),
            }
        }
        // ---------- IpAddress Types ----------
        Type::INET => Ok(
            composite_field_postgres_to_py::<Option<IpAddr>>(type_, buf, is_simple)?
                .into_py_any(py)?,
        ),
        // Convert JSON/JSONB into Serde Value, then into list or dict
        Type::JSONB | Type::JSON => {
            let db_json = composite_field_postgres_to_py::<Option<Value>>(type_, buf, is_simple)?;

            match db_json {
                Some(value) => Ok(build_python_from_serde_value(py, value)?),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        // Convert MACADDR into inner type for macaddr6, then into str
        Type::MACADDR => {
            let macaddr_ =
                composite_field_postgres_to_py::<Option<RustMacAddr6>>(type_, buf, is_simple)?;
            if let Some(macaddr_) = macaddr_ {
                Ok(macaddr_.inner().to_string().into_py_any(py)?)
            } else {
                Ok(py.None().into_py_any(py)?)
            }
        }
        Type::MACADDR8 => {
            let macaddr_ =
                composite_field_postgres_to_py::<Option<RustMacAddr8>>(type_, buf, is_simple)?;
            if let Some(macaddr_) = macaddr_ {
                Ok(macaddr_.inner().to_string().into_py_any(py)?)
            } else {
                Ok(py.None().into_py_any(py)?)
            }
        }
        Type::NUMERIC => {
            if let Some(numeric_) =
                composite_field_postgres_to_py::<Option<Decimal>>(type_, buf, is_simple)?
            {
                return Ok(InnerDecimal(numeric_).into_py_any(py)?);
            }
            Ok(py.None().into_py_any(py)?)
        }
        // ---------- Geo Types ----------
        Type::POINT => {
            let point_ =
                composite_field_postgres_to_py::<Option<RustPoint>>(type_, buf, is_simple)?;

            match point_ {
                Some(point_) => Ok(point_.into_pyobject(py)?.unbind()),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        Type::BOX => {
            let box_ = composite_field_postgres_to_py::<Option<RustRect>>(type_, buf, is_simple)?;

            match box_ {
                Some(box_) => Ok(box_.into_pyobject(py)?.unbind()),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        Type::PATH => {
            let path_ =
                composite_field_postgres_to_py::<Option<RustLineString>>(type_, buf, is_simple)?;

            match path_ {
                Some(path_) => Ok(path_.into_pyobject(py)?.unbind()),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        Type::LINE => {
            let line_ = composite_field_postgres_to_py::<Option<Line>>(type_, buf, is_simple)?;

            match line_ {
                Some(line_) => Ok(line_.into_pyobject(py)?.unbind()),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        Type::LSEG => {
            let lseg_ =
                composite_field_postgres_to_py::<Option<RustLineSegment>>(type_, buf, is_simple)?;

            match lseg_ {
                Some(lseg_) => Ok(lseg_.into_pyobject(py)?.unbind()),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        Type::CIRCLE => {
            let circle_ = composite_field_postgres_to_py::<Option<Circle>>(type_, buf, is_simple)?;

            match circle_ {
                Some(circle_) => Ok(circle_.into_pyobject(py)?.unbind()),
                None => Ok(py.None().into_py_any(py)?),
            }
        }
        Type::INTERVAL => {
            let interval =
                composite_field_postgres_to_py::<Option<Interval>>(type_, buf, is_simple)?;
            if let Some(interval) = interval {
                return Ok(InnerInterval(interval).into_py_any(py)?);
            }
            Ok(py.None())
        }
        // ---------- Array Text Types ----------
        Type::BOOL_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<bool>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        Type::OID_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i32>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of TEXT or VARCHAR into Vec<String>, then into list[str]
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::XML_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<String>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // ---------- Array Integer Types ----------
        // Convert ARRAY of SmallInt into Vec<i16>, then into list[int]
        Type::INT2_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i16>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of Integer into Vec<i32>, then into list[int]
        Type::INT4_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i32>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of BigInt into Vec<i64>, then into list[int]
        Type::INT8_ARRAY | Type::MONEY_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<i64>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of Float4 into Vec<f32>, then into list[float]
        Type::FLOAT4_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<f32>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of Float8 into Vec<f64>, then into list[float]
        Type::FLOAT8_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<f64>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of Date into Vec<NaiveDate>, then into list[datetime.date]
        Type::DATE_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<NaiveDate>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of Time into Vec<NaiveTime>, then into list[datetime.date]
        Type::TIME_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<NaiveTime>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of TIMESTAMP into Vec<NaiveDateTime>, then into list[datetime.date]
        Type::TIMESTAMP_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<NaiveDateTime>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of TIMESTAMPTZ into Vec<DateTime<FixedOffset>>, then into list[datetime.date]
        Type::TIMESTAMPTZ_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<DateTime<FixedOffset>>>>(
                type_, buf, is_simple,
            )?,
        )
        .into_py_any(py)?),
        // Convert ARRAY of UUID into Vec<Array<InternalUuid>>, then into list[UUID]
        Type::UUID_ARRAY => {
            let uuid_array = composite_field_postgres_to_py::<Option<Array<InternalUuid>>>(
                type_, buf, is_simple,
            )?;
            Ok(postgres_array_to_py(py, uuid_array).into_py_any(py)?)
        }
        // Convert ARRAY of INET into Vec<INET>, then into list[IPv4Address | IPv6Address]
        Type::INET_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<IpAddr>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        Type::JSONB_ARRAY | Type::JSON_ARRAY => {
            let db_json_array = composite_field_postgres_to_py::<Option<Array<InternalSerdeValue>>>(
                type_, buf, is_simple,
            )?;
            Ok(postgres_array_to_py(py, db_json_array).into_py_any(py)?)
        }
        Type::NUMERIC_ARRAY => Ok(postgres_array_to_py(
            py,
            composite_field_postgres_to_py::<Option<Array<InnerDecimal>>>(type_, buf, is_simple)?,
        )
        .into_py_any(py)?),
        // ---------- Array Geo Types ----------
        Type::POINT_ARRAY => {
            let point_array_ =
                composite_field_postgres_to_py::<Option<Array<RustPoint>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, point_array_).into_py_any(py)?)
        }
        Type::BOX_ARRAY => {
            let box_array_ =
                composite_field_postgres_to_py::<Option<Array<RustRect>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, box_array_).into_py_any(py)?)
        }
        Type::PATH_ARRAY => {
            let path_array_ = composite_field_postgres_to_py::<Option<Array<RustLineString>>>(
                type_, buf, is_simple,
            )?;

            Ok(postgres_array_to_py(py, path_array_).into_py_any(py)?)
        }
        Type::LINE_ARRAY => {
            let line_array_ =
                composite_field_postgres_to_py::<Option<Array<Line>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, line_array_).into_py_any(py)?)
        }
        Type::LSEG_ARRAY => {
            let lseg_array_ = composite_field_postgres_to_py::<Option<Array<RustLineSegment>>>(
                type_, buf, is_simple,
            )?;

            Ok(postgres_array_to_py(py, lseg_array_).into_py_any(py)?)
        }
        Type::CIRCLE_ARRAY => {
            let circle_array_ =
                composite_field_postgres_to_py::<Option<Array<Circle>>>(type_, buf, is_simple)?;

            Ok(postgres_array_to_py(py, circle_array_).into_py_any(py)?)
        }
        Type::INTERVAL_ARRAY => {
            let interval_array_ = composite_field_postgres_to_py::<Option<Array<InnerInterval>>>(
                type_, buf, is_simple,
            )?;

            Ok(postgres_array_to_py(py, interval_array_).into_py_any(py)?)
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
) -> PSQLPyResult<Py<PyAny>> {
    if type_.name() == "vector" {
        let vector = composite_field_postgres_to_py::<Option<PgVector>>(type_, buf, is_simple)?;
        match vector {
            Some(real_vector) => {
                return Ok(real_vector.to_vec().into_py_any(py)?);
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
) -> PSQLPyResult<Py<PyAny>> {
    let result_py_dict: Bound<'_, PyDict> = PyDict::new(py);

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
                    postgres_bytes_to_py(py, field.type_(), buf, false)?,
                )?;
            }
            Kind::Enum(_) => {
                result_py_dict.set_item(
                    field.name(),
                    postgres_bytes_to_py(py, &Type::VARCHAR, buf, false)?,
                )?;
            }
            _ => {
                let (_, tail) = buf.split_at(4_usize);
                *buf = tail;
                result_py_dict.set_item(
                    field.name(),
                    raw_bytes_data_process(py, buf, field.name(), field.type_(), custom_decoders)?,
                )?;
            }
        }
    }

    Ok(result_py_dict.into_py_any(py)?)
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
) -> PSQLPyResult<Py<PyAny>> {
    if let Some(custom_decoders) = custom_decoders {
        let py_encoder_func = custom_decoders
            .bind(py)
            .get_item(column_name.to_lowercase());

        if let Ok(Some(py_encoder_func)) = py_encoder_func {
            return Ok(py_encoder_func
                .call1((PyBytes::new(py, raw_bytes_data),))?
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
) -> PSQLPyResult<Py<PyAny>> {
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
