use postgres_types::{FromSql, Type};
use pyo3::{types::PyAnyMethods, Bound, IntoPyObject, PyAny, PyObject, Python, ToPyObject};
use rust_decimal::Decimal;

use crate::value_converter::consts::get_decimal_cls;

pub struct InnerDecimal(pub Decimal);

impl ToPyObject for InnerDecimal {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dec_cls = get_decimal_cls(py).expect("failed to load decimal.Decimal");
        let ret = dec_cls
            .call1((self.0.to_string(),))
            .expect("failed to call decimal.Decimal(value)");
        ret.to_object(py)
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
