use pg_interval::Interval;
use postgres_types::{FromSql, Type};
use pyo3::{
    types::{PyAnyMethods, PyDict, PyDictMethods},
    PyObject, Python, ToPyObject,
};

use crate::value_converter::consts::get_timedelta_cls;

pub struct InnerInterval(pub Interval);

impl ToPyObject for InnerInterval {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let td_cls = get_timedelta_cls(py).expect("failed to load datetime.timedelta");
        let pydict = PyDict::new_bound(py);
        let months = self.0.months * 30;
        let _ = pydict.set_item("days", self.0.days + months);
        let _ = pydict.set_item("microseconds", self.0.microseconds);
        let ret = td_cls
            .call((), Some(&pydict))
            .expect("failed to call datetime.timedelta(days=<>, microseconds=<>)");
        ret.to_object(py)
    }
}

impl<'a> FromSql<'a> for InnerInterval {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(InnerInterval(<Interval as FromSql>::from_sql(ty, raw)?))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
