use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::RwLock};

use pyo3::{
    sync::GILOnceCell,
    types::{PyAnyMethods, PyType},
    Bound, Py, PyResult, Python,
};

pub static KWARGS_PARAMS_REGEXP: &str = r"\$\(([^)]+)\)p";

pub static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
pub static TIMEDELTA_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
pub static KWARGS_QUERYSTRINGS: Lazy<RwLock<HashMap<String, (String, Vec<String>)>>> =
    Lazy::new(|| RwLock::new(Default::default()));

pub fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    DECIMAL_CLS
        .get_or_try_init(py, || {
            let type_object = py.import("decimal")?.getattr("Decimal")?.downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
}

pub fn get_timedelta_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    TIMEDELTA_CLS
        .get_or_try_init(py, || {
            let type_object = py
                .import("datetime")?
                .getattr("timedelta")?
                .downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
}
