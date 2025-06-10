use postgres_types::{FromSql, Type};
use pyo3::{types::PyAnyMethods, Bound, IntoPyObject, IntoPyObjectExt, PyAny, Python};
use rust_decimal::Decimal;

use crate::{
    exceptions::rust_errors::RustPSQLDriverError, value_converter::consts::get_decimal_cls,
};

#[derive(Clone)]
pub struct InnerDecimal(pub Decimal);

impl<'py> IntoPyObject<'py> for InnerDecimal {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = RustPSQLDriverError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dec_cls = get_decimal_cls(py).expect("failed to load decimal.Decimal");
        let ret = dec_cls
            .call1((self.0.to_string(),))
            .expect("failed to call decimal.Decimal(value)");
        match ret.into_py_any(py) {
            Ok(result) => Ok(result.bind(py).clone()),
            Err(_) => Err(RustPSQLDriverError::RustToPyValueConversionError(
                "Cannot convert Rust decimal to Python one".into(),
            )),
        }
    }
}

impl<'a> FromSql<'a> for InnerDecimal {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InnerDecimal(<Decimal as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
