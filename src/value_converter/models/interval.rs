use pg_interval::Interval;
use postgres_types::{FromSql, Type};
use pyo3::{
    types::{PyAnyMethods, PyDict, PyDictMethods},
    Bound, IntoPyObject, PyAny, Python,
};

use crate::{
    exceptions::rust_errors::RustPSQLDriverError, value_converter::consts::get_timedelta_cls,
};

pub struct InnerInterval(pub Interval);

impl<'py> IntoPyObject<'py> for InnerInterval {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = RustPSQLDriverError;
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let td_cls = get_timedelta_cls(py).expect("failed to load datetime.timedelta");
        let pydict = PyDict::new(py);
        let months = self.0.months * 30;
        let _ = pydict.set_item("days", self.0.days + months);
        let _ = pydict.set_item("microseconds", self.0.microseconds);
        let ret = td_cls
            .call((), Some(&pydict))
            .expect("failed to call datetime.timedelta(days=<>, microseconds=<>)");
        match ret.into_pyobject(py) {
            Ok(res) => Ok(res),
            Err(_) => unreachable!(),
        }
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
