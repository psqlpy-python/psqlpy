use pyo3::{
    pyclass, pyfunction, pymethods,
    types::{PyDict, PyDictMethods, PyModule, PyModuleMethods, PyTuple},
    wrap_pyfunction, Bound, IntoPyObject, Py, PyAny, PyResult, Python,
};

use crate::exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError};

#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn tuple_row(py: Python<'_>, dict_: Py<PyAny>) -> PSQLPyResult<Py<PyAny>> {
    let dict_ = dict_.downcast_bound::<PyDict>(py).map_err(|_| {
        RustPSQLDriverError::RustToPyValueConversionError(
            "as_tuple accepts only dict as a parameter".into(),
        )
    })?;
    match PyTuple::new(py, dict_.items())?.into_pyobject(py) {
        Ok(x) => Ok(x.unbind().into_any()),
        _ => unreachable!(),
    }
}

#[pyclass]
#[allow(non_camel_case_types)]
struct class_row(Py<PyAny>);

#[pymethods]
impl class_row {
    #[new]
    fn construct_class(class_: Py<PyAny>) -> Self {
        Self(class_)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn __call__(&self, py: Python<'_>, dict_: Py<PyAny>) -> PSQLPyResult<Py<PyAny>> {
        let dict_ = dict_.downcast_bound::<PyDict>(py).map_err(|_| {
            RustPSQLDriverError::RustToPyValueConversionError(
                "as_tuple accepts only dict as a parameter".into(),
            )
        })?;
        Ok(self.0.call(py, (), Some(dict_))?)
    }
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::missing_errors_doc)]
pub fn row_factories_module(_py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add_function(wrap_pyfunction!(tuple_row, pymod)?)?;
    pymod.add_class::<class_row>()?;
    Ok(())
}
