use std::net::IpAddr;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use pg_interval::Interval;
use postgres_types::Type;
use pyo3::{
    types::{PyAnyMethods, PyDateTime, PyDelta, PyDict},
    Bound, PyAny,
};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    extra_types::{self, PythonDecimal, PythonUUID},
    value_converter::{
        additional_types::NonePyType,
        from_python::{extract_datetime_from_python_object_attrs, py_sequence_into_postgres_array},
        models::serde_value::build_serde_value,
        traits::{ToPythonDTO, ToPythonDTOArray},
    },
};

use super::{enums::PythonDTO, funcs::array_type_to_single_type};

impl ToPythonDTO for NonePyType {
    fn to_python_dto(_python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        Ok(PythonDTO::PyNone)
    }
}

macro_rules! construct_simple_type_converter {
    ($match_type:ty, $kind:path) => {
        impl ToPythonDTO for $match_type {
            fn to_python_dto(python_param: &Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
                Ok($kind(python_param.extract::<$match_type>()?))
            }
        }
    };
}

construct_simple_type_converter!(bool, PythonDTO::PyBool);
construct_simple_type_converter!(Vec<u8>, PythonDTO::PyBytes);
construct_simple_type_converter!(String, PythonDTO::PyString);
construct_simple_type_converter!(f32, PythonDTO::PyFloat32);
construct_simple_type_converter!(f64, PythonDTO::PyFloat64);
construct_simple_type_converter!(i16, PythonDTO::PyIntI16);
construct_simple_type_converter!(i32, PythonDTO::PyIntI32);
construct_simple_type_converter!(i64, PythonDTO::PyIntI64);
construct_simple_type_converter!(NaiveDate, PythonDTO::PyDate);
construct_simple_type_converter!(NaiveTime, PythonDTO::PyTime);

impl ToPythonDTO for PyDateTime {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        let timestamp_tz = python_param.extract::<DateTime<FixedOffset>>();
        if let Ok(pydatetime_tz) = timestamp_tz {
            return Ok(PythonDTO::PyDateTimeTz(pydatetime_tz));
        }

        let timestamp_no_tz = python_param.extract::<NaiveDateTime>();
        if let Ok(pydatetime_no_tz) = timestamp_no_tz {
            return Ok(PythonDTO::PyDateTime(pydatetime_no_tz));
        }

        let timestamp_tz = extract_datetime_from_python_object_attrs(python_param);
        if let Ok(pydatetime_tz) = timestamp_tz {
            return Ok(PythonDTO::PyDateTimeTz(pydatetime_tz));
        }

        return Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Can not convert you datetime to rust type".into(),
        ));
    }
}

impl ToPythonDTO for PyDelta {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        let duration = python_param.extract::<chrono::Duration>()?;
        if let Some(interval) = Interval::from_duration(duration) {
            return Ok(PythonDTO::PyInterval(interval));
        }
        return Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Cannot convert timedelta from Python to inner Rust type.".to_string(),
        ));
    }
}

impl ToPythonDTO for PyDict {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        let serde_value = build_serde_value(python_param)?;

        return Ok(PythonDTO::PyJsonb(serde_value));
    }
}

macro_rules! construct_extra_type_converter {
    ($match_type:ty, $kind:path) => {
        impl ToPythonDTO for $match_type {
            fn to_python_dto(python_param: &Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
                Ok($kind(python_param.extract::<$match_type>()?.inner()))
            }
        }
    };
}

construct_extra_type_converter!(extra_types::Text, PythonDTO::PyText);
construct_extra_type_converter!(extra_types::VarChar, PythonDTO::PyVarChar);
construct_extra_type_converter!(extra_types::SmallInt, PythonDTO::PyIntI16);
construct_extra_type_converter!(extra_types::Integer, PythonDTO::PyIntI32);
construct_extra_type_converter!(extra_types::BigInt, PythonDTO::PyIntI64);
construct_extra_type_converter!(extra_types::Float32, PythonDTO::PyFloat32);
construct_extra_type_converter!(extra_types::Float64, PythonDTO::PyFloat64);
construct_extra_type_converter!(extra_types::Money, PythonDTO::PyMoney);
construct_extra_type_converter!(extra_types::JSONB, PythonDTO::PyJsonb);
construct_extra_type_converter!(extra_types::JSON, PythonDTO::PyJson);
construct_extra_type_converter!(extra_types::MacAddr6, PythonDTO::PyMacAddr6);
construct_extra_type_converter!(extra_types::MacAddr8, PythonDTO::PyMacAddr8);
construct_extra_type_converter!(extra_types::Point, PythonDTO::PyPoint);
construct_extra_type_converter!(extra_types::Box, PythonDTO::PyBox);
construct_extra_type_converter!(extra_types::Path, PythonDTO::PyPath);
construct_extra_type_converter!(extra_types::Line, PythonDTO::PyLine);
construct_extra_type_converter!(extra_types::LineSegment, PythonDTO::PyLineSegment);
construct_extra_type_converter!(extra_types::Circle, PythonDTO::PyCircle);
construct_extra_type_converter!(extra_types::PgVector, PythonDTO::PyPgVector);
construct_extra_type_converter!(extra_types::CustomType, PythonDTO::PyCustomType);

impl ToPythonDTO for PythonDecimal {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        Ok(PythonDTO::PyDecimal(Decimal::from_str_exact(
            python_param.str()?.extract::<&str>()?,
        )?))
    }
}

impl ToPythonDTO for PythonUUID {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        Ok(PythonDTO::PyUUID(Uuid::parse_str(
            python_param.str()?.extract::<&str>()?,
        )?))
    }
}

impl ToPythonDTOArray for extra_types::PythonArray {
    fn to_python_dto(
        python_param: &pyo3::Bound<'_, PyAny>,
        array_type: Type,
    ) -> PSQLPyResult<PythonDTO> {
        let elem_type = array_type_to_single_type(&array_type);
        Ok(PythonDTO::PyArray(
            py_sequence_into_postgres_array(python_param, &elem_type)?,
            array_type,
        ))
    }
}

impl ToPythonDTO for IpAddr {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        if let Ok(id_address) = python_param.extract::<IpAddr>() {
            return Ok(PythonDTO::PyIpAddress(id_address));
        }

        Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Parameter passed to IpAddr is incorrect.".to_string(),
        ))
    }
}

impl ToPythonDTO for extra_types::PythonEnum {
    fn to_python_dto(python_param: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
        if let Ok(value_attr) = python_param.getattr("value") {
            if let Ok(possible_string) = value_attr.extract::<String>() {
                return Ok(PythonDTO::PyString(possible_string));
            }
        }

        Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Cannot convert Enum to inner type".into(),
        ))
    }
}

macro_rules! construct_array_type_converter {
    ($match_type:ty) => {
        impl ToPythonDTO for $match_type {
            fn to_python_dto(python_param: &Bound<'_, PyAny>) -> PSQLPyResult<PythonDTO> {
                python_param
                    .extract::<$match_type>()?
                    ._convert_to_python_dto(&Self::element_type())
            }
        }
    };
}

construct_array_type_converter!(extra_types::BoolArray);
construct_array_type_converter!(extra_types::UUIDArray);
construct_array_type_converter!(extra_types::VarCharArray);
construct_array_type_converter!(extra_types::TextArray);
construct_array_type_converter!(extra_types::Int16Array);
construct_array_type_converter!(extra_types::Int32Array);
construct_array_type_converter!(extra_types::Int64Array);
construct_array_type_converter!(extra_types::Float32Array);
construct_array_type_converter!(extra_types::Float64Array);
construct_array_type_converter!(extra_types::MoneyArray);
construct_array_type_converter!(extra_types::IpAddressArray);
construct_array_type_converter!(extra_types::JSONBArray);
construct_array_type_converter!(extra_types::JSONArray);
construct_array_type_converter!(extra_types::DateArray);
construct_array_type_converter!(extra_types::TimeArray);
construct_array_type_converter!(extra_types::DateTimeArray);
construct_array_type_converter!(extra_types::DateTimeTZArray);
construct_array_type_converter!(extra_types::MacAddr6Array);
construct_array_type_converter!(extra_types::MacAddr8Array);
construct_array_type_converter!(extra_types::NumericArray);
construct_array_type_converter!(extra_types::PointArray);
construct_array_type_converter!(extra_types::BoxArray);
construct_array_type_converter!(extra_types::PathArray);
construct_array_type_converter!(extra_types::LineArray);
construct_array_type_converter!(extra_types::LsegArray);
construct_array_type_converter!(extra_types::CircleArray);
construct_array_type_converter!(extra_types::IntervalArray);
