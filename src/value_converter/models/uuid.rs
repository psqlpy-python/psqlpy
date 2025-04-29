use postgres_types::FromSql;
use uuid::Uuid;

use pyo3::{
    types::PyAnyMethods, Bound, FromPyObject, PyAny, PyObject, PyResult, Python, ToPyObject,
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

impl ToPyObject for InternalUuid {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.to_string().as_str().to_object(py)
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
