use postgres_types::FromSql;
use uuid::Uuid;

use pyo3::{
    types::{PyAnyMethods, PyString},
    Bound, FromPyObject, IntoPyObject, PyAny, PyResult, Python,
};
use tokio_postgres::types::Type;

use crate::exceptions::rust_errors::RustPSQLDriverError;

/// Struct for Uuid.
///
/// We use custom struct because we need to implement external traits
/// to it.
#[derive(Clone, Copy)]
pub struct InternalUuid(Uuid);

impl<'a> FromPyObject<'a> for InternalUuid {
    fn extract_bound(obj: &Bound<'a, PyAny>) -> PyResult<Self> {
        let uuid_value = Uuid::parse_str(obj.str()?.extract::<&str>()?).map_err(|_| {
            RustPSQLDriverError::PyToRustValueConversionError(
                "Cannot convert UUID Array to inner rust type, check you parameters.".into(),
            )
        })?;
        Ok(InternalUuid(uuid_value))
    }
}

impl<'py> IntoPyObject<'py> for InternalUuid {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = RustPSQLDriverError;

    /// Performs the conversion.
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self.0.to_string().as_str().into_pyobject(py) {
            Ok(result) => Ok(result),
            _ => unreachable!(),
        }
    }
}

impl<'a> FromSql<'a> for InternalUuid {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InternalUuid(<Uuid as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
