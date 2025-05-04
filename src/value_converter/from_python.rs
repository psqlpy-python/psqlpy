use chrono::{self, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;
use geo_types::{coord, Coord};
use itertools::Itertools;
use postgres_array::{Array, Dimension};
use postgres_types::Type;
use std::net::IpAddr;

use pyo3::{
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyFloat, PyInt, PyList,
        PySequence, PySet, PyString, PyTime, PyTuple, PyTypeMethods,
    },
    Bound, Py, PyAny, Python,
};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    extra_types::{self},
    value_converter::{dto::enums::PythonDTO, utils::extract_value_from_python_object_or_raise},
};

use super::{
    additional_types::NonePyType,
    traits::{ToPythonDTO, ToPythonDTOArray},
};

/// Convert single python parameter to `PythonDTO` enum.
///
/// # Errors
///
/// May return Err Result if python type doesn't have support yet
/// or value of the type is incorrect.
#[allow(clippy::too_many_lines)]
pub fn from_python_untyped(parameter: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
    if parameter.is_none() {
        return <NonePyType as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyBool>() {
        return <bool as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyBytes>() {
        return <Vec<u8> as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Text>() {
        return <extra_types::Text as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::VarChar>() {
        return <extra_types::VarChar as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyString>() {
        return <String as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyFloat>() {
        return <f64 as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Float32>() {
        return <extra_types::Float32 as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Float64>() {
        return <extra_types::Float64 as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::SmallInt>() {
        return <extra_types::SmallInt as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Integer>() {
        return <extra_types::Integer as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::BigInt>() {
        return <extra_types::BigInt as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Money>() {
        return <extra_types::Money as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyInt>() {
        return <i32 as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyDateTime>() {
        return <PyDateTime as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyDate>() {
        return <NaiveDate as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyTime>() {
        return <NaiveTime as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyDelta>() {
        return <PyDelta as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyList>() | parameter.is_instance_of::<PyTuple>() {
        return <extra_types::PythonArray as ToPythonDTOArray>::to_python_dto(parameter, Type::ANY);
    }

    if parameter.is_instance_of::<PyDict>() {
        return <PyDict as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::JSONB>() {
        return <extra_types::JSONB as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::JSON>() {
        return <extra_types::JSON as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::MacAddr6>() {
        return <extra_types::MacAddr6 as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::MacAddr8>() {
        return <extra_types::MacAddr8 as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Point>() {
        return <extra_types::Point as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Box>() {
        return <extra_types::Box as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Path>() {
        return <extra_types::Path as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Line>() {
        return <extra_types::Line as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::LineSegment>() {
        return <extra_types::LineSegment as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Circle>() {
        return <extra_types::Circle as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.get_type().name()? == "UUID" {
        return <extra_types::PythonUUID as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.get_type().name()? == "decimal.Decimal"
        || parameter.get_type().name()? == "Decimal"
    {
        return <extra_types::PythonDecimal as ToPythonDTO>::to_python_dto(parameter);
    }

    if let Ok(converted_array) = from_python_array_typed(parameter) {
        return Ok(converted_array);
    }

    if parameter.is_instance_of::<extra_types::PgVector>() {
        return <extra_types::PgVector as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.extract::<IpAddr>().is_ok() {
        return <IpAddr as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.getattr("value").is_ok() {
        return <extra_types::PythonEnum as ToPythonDTO>::to_python_dto(parameter);
    }

    Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
        "Can not covert you type {parameter} into inner one",
    )))
}

/// Convert single python parameter to `PythonDTO` enum.
///
/// # Errors
///
/// May return Err Result if python type doesn't have support yet
/// or value of the type is incorrect.
#[allow(clippy::too_many_lines)]
pub fn from_python_typed(
    parameter: &pyo3::Bound<'_, PyAny>,
    type_: &Type,
) -> PSQLPyResult<PythonDTO> {
    if parameter.is_instance_of::<extra_types::CustomType>() {
        return <extra_types::CustomType as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_none() {
        return <NonePyType as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.get_type().name()? == "UUID" {
        return <extra_types::PythonUUID as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.get_type().name()? == "decimal.Decimal"
        || parameter.get_type().name()? == "Decimal"
    {
        return <extra_types::PythonDecimal as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<PyList>() | parameter.is_instance_of::<PyTuple>() {
        return <extra_types::PythonArray as ToPythonDTOArray>::to_python_dto(
            parameter,
            type_.clone(),
        );
    }

    if let Ok(converted_array) = from_python_array_typed(parameter) {
        return Ok(converted_array);
    }

    match *type_ {
        Type::BYTEA => return <Vec<u8> as ToPythonDTO>::to_python_dto(parameter),
        Type::TEXT => {
            if parameter.is_instance_of::<extra_types::Text>() {
                return <extra_types::Text as ToPythonDTO>::to_python_dto(parameter);
            }
            return <String as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::VARCHAR => {
            if parameter.is_instance_of::<extra_types::VarChar>() {
                return <extra_types::VarChar as ToPythonDTO>::to_python_dto(parameter);
            }
            return <String as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::XML => return <String as ToPythonDTO>::to_python_dto(parameter),
        Type::BOOL => return <bool as ToPythonDTO>::to_python_dto(parameter),
        Type::INT2 => {
            if parameter.is_instance_of::<extra_types::SmallInt>() {
                return <extra_types::SmallInt as ToPythonDTO>::to_python_dto(parameter);
            }
            return <i16 as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::INT4 => {
            if parameter.is_instance_of::<extra_types::Integer>() {
                return <extra_types::Integer as ToPythonDTO>::to_python_dto(parameter);
            }
            return <i32 as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::INT8 => {
            if parameter.is_instance_of::<extra_types::BigInt>() {
                return <extra_types::BigInt as ToPythonDTO>::to_python_dto(parameter);
            }
            return <i64 as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::MONEY => {
            if parameter.is_instance_of::<extra_types::Money>() {
                return <extra_types::Money as ToPythonDTO>::to_python_dto(parameter);
            }
            return <i64 as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::FLOAT4 => {
            if parameter.is_instance_of::<extra_types::Float32>() {
                return <extra_types::Float32 as ToPythonDTO>::to_python_dto(parameter);
            }
            return <f32 as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::FLOAT8 => {
            if parameter.is_instance_of::<extra_types::Float64>() {
                return <extra_types::Float64 as ToPythonDTO>::to_python_dto(parameter);
            }
            return <f64 as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::INET => return <IpAddr as ToPythonDTO>::to_python_dto(parameter),
        Type::DATE => return <NaiveDate as ToPythonDTO>::to_python_dto(parameter),
        Type::TIME => return <NaiveTime as ToPythonDTO>::to_python_dto(parameter),
        Type::TIMESTAMP | Type::TIMESTAMPTZ => {
            return <PyDateTime as ToPythonDTO>::to_python_dto(parameter)
        }
        Type::INTERVAL => return <PyDelta as ToPythonDTO>::to_python_dto(parameter),
        Type::JSONB => {
            if parameter.is_instance_of::<extra_types::JSONB>() {
                return <extra_types::JSONB as ToPythonDTO>::to_python_dto(parameter);
            }

            return <PyDict as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::JSON => {
            if parameter.is_instance_of::<extra_types::JSON>() {
                return <extra_types::JSON as ToPythonDTO>::to_python_dto(parameter);
            }

            return <PyDict as ToPythonDTO>::to_python_dto(parameter);
        }
        Type::MACADDR => return <extra_types::MacAddr6 as ToPythonDTO>::to_python_dto(parameter),
        Type::MACADDR8 => return <extra_types::MacAddr8 as ToPythonDTO>::to_python_dto(parameter),
        Type::POINT => return <extra_types::Point as ToPythonDTO>::to_python_dto(parameter),
        Type::BOX => return <extra_types::Box as ToPythonDTO>::to_python_dto(parameter),
        Type::PATH => return <extra_types::Path as ToPythonDTO>::to_python_dto(parameter),
        Type::LINE => return <extra_types::Line as ToPythonDTO>::to_python_dto(parameter),
        Type::LSEG => return <extra_types::LineSegment as ToPythonDTO>::to_python_dto(parameter),
        Type::CIRCLE => return <extra_types::Circle as ToPythonDTO>::to_python_dto(parameter),
        _ => {}
    }

    if let Ok(converted_value) = from_python_untyped(parameter) {
        return Ok(converted_value);
    }

    Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
        "Can not covert you type {parameter} into {type_}",
    )))
}

fn from_python_array_typed(parameter: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
    if parameter.is_instance_of::<extra_types::BoolArray>() {
        return <extra_types::BoolArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::UUIDArray>() {
        return <extra_types::UUIDArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::VarCharArray>() {
        return <extra_types::VarCharArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::TextArray>() {
        return <extra_types::TextArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Int16Array>() {
        return <extra_types::Int16Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Int32Array>() {
        return <extra_types::Int32Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Int64Array>() {
        return <extra_types::Int64Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Float32Array>() {
        return <extra_types::Float32Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::Float64Array>() {
        return <extra_types::Float64Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::MoneyArray>() {
        return <extra_types::MoneyArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::IpAddressArray>() {
        return <extra_types::IpAddressArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::JSONBArray>() {
        return <extra_types::JSONBArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::JSONArray>() {
        return <extra_types::JSONArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::DateArray>() {
        return <extra_types::DateArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::TimeArray>() {
        return <extra_types::TimeArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::DateTimeArray>() {
        return <extra_types::DateTimeArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::DateTimeTZArray>() {
        return <extra_types::DateTimeTZArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::MacAddr6Array>() {
        return <extra_types::MacAddr6Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::MacAddr8Array>() {
        return <extra_types::MacAddr8Array as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::NumericArray>() {
        return <extra_types::NumericArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::PointArray>() {
        return <extra_types::PointArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::BoxArray>() {
        return <extra_types::BoxArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::PathArray>() {
        return <extra_types::PathArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::LineArray>() {
        return <extra_types::LineArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::LsegArray>() {
        return <extra_types::LsegArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::CircleArray>() {
        return <extra_types::CircleArray as ToPythonDTO>::to_python_dto(parameter);
    }

    if parameter.is_instance_of::<extra_types::IntervalArray>() {
        return <extra_types::IntervalArray as ToPythonDTO>::to_python_dto(parameter);
    }

    Err(RustPSQLDriverError::PyToRustValueConversionError(format!(
        "Cannot convert parameter in extra types Array",
    )))
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
pub fn extract_datetime_from_python_object_attrs(
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
    type_: &Type,
) -> PSQLPyResult<Array<PythonDTO>> {
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

        let first_seq_elem = py_seq.try_iter()?.next();
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

    let array_data = py_sequence_into_flat_vec(parameter, type_)?;
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
    type_: &Type,
) -> PSQLPyResult<Vec<PythonDTO>> {
    let py_seq = parameter.downcast::<PySequence>().map_err(|_| {
        RustPSQLDriverError::PyToRustValueConversionError(
            "PostgreSQL ARRAY type can be made only from python Sequence".into(),
        )
    })?;

    let mut final_vec: Vec<PythonDTO> = vec![];

    for seq_elem in py_seq.try_iter()? {
        let ok_seq_elem = seq_elem?;

        // Check for the string because it's sequence too,
        // and in the most cases it should be array type, not new dimension.
        if ok_seq_elem.is_instance_of::<PyString>() {
            final_vec.push(from_python_typed(&ok_seq_elem, type_)?);
            continue;
        }

        let possible_next_seq = ok_seq_elem.downcast::<PySequence>();

        if let Ok(next_seq) = possible_next_seq {
            let mut next_vec = py_sequence_into_flat_vec(next_seq, type_)?;
            final_vec.append(&mut next_vec);
        } else {
            final_vec.push(from_python_typed(&ok_seq_elem, type_)?);
            continue;
        }
    }

    Ok(final_vec)
}

/// Convert two python parameters(x and y) to Coord from `geo_type`.
/// Also it checks that passed values is int or float.
///
/// # Errors
///
/// May return error if cannot convert Python type into Rust one.
/// May return error if parameters type isn't correct.
fn convert_py_to_rust_coord_values(parameters: Vec<Py<PyAny>>) -> PSQLPyResult<Vec<f64>> {
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

            let python_dto = from_python_untyped(parameter_bind)?;
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
) -> PSQLPyResult<Vec<Coord>> {
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
) -> PSQLPyResult<Vec<f64>> {
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
fn py_sequence_to_rust(bind_parameters: &Bound<PyAny>) -> PSQLPyResult<Vec<Py<PyAny>>> {
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
