use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;
use geo_types::{coord, Coord};
use itertools::Itertools;
use pg_interval::Interval;
use postgres_array::{Array, Dimension};
use postgres_types::{Field, FromSql, Kind, Type};
use rust_decimal::Decimal;
use serde_json::{json, Map, Value};
use std::net::IpAddr;
use tokio_postgres::{Column, Row};
use uuid::Uuid;

use pyo3::{
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyDictMethods, PyFloat,
        PyInt, PyList, PyListMethods, PyMapping, PySequence, PySet, PyString, PyTime, PyTuple,
        PyTypeMethods,
    },
    Bound, FromPyObject, IntoPy, Py, PyAny, Python, ToPyObject,
};

use crate::{
    additional_types::{
        Circle, Line, RustLineSegment, RustLineString, RustMacAddr6, RustMacAddr8, RustPoint,
        RustRect,
    },
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    extra_types::{self},
    value_converter::{consts::KWARGS_QUERYSTRINGS, models::dto::PythonDTO},
};

use pgvector::Vector as PgVector;

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
