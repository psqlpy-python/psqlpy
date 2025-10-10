use pyo3::{
    sync::GILOnceCell,
    types::{PyAnyMethods, PyType},
    Bound, Py, PyResult, Python,
};

pub static KWARGS_PARAMS_REGEXP: &str = r"\$\(([^)]+)\)p";

pub static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
pub static TIMEDELTA_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

#[allow(clippy::missing_errors_doc)]
pub fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    DECIMAL_CLS
        .get_or_try_init(py, || {
            let type_object = py.import("decimal")?.getattr("Decimal")?.downcast_into()?;
            Ok(type_object.unbind())
        })
        .map(|ty| ty.bind(py))
}

#[allow(clippy::missing_errors_doc)]
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
