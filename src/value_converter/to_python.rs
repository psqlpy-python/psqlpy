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
    types::{
        PyAnyMethods, PyBytes, PyDict, PyDictMethods, PyList, PyListMethods, PySet, PyString,
        PyTuple,
    },
    Bound, IntoPy, Py, PyAny, Python, ToPyObject,
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
) -> PSQLPyResult<Py<PyAny>> {
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
) -> PSQLPyResult<Py<PyAny>> {
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
) -> PSQLPyResult<Py<PyAny>> {
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
) -> PSQLPyResult<Py<PyAny>> {
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
