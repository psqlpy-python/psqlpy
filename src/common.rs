use std::future::Future;

use pyo3::{types::PyModule, IntoPy, PyAny, PyObject, PyResult, Python};

use crate::exceptions::rust_errors::RustPSQLDriverPyResult;

/// Add new module to the parent one.
///
/// You can find out more information from this issue
/// <https://github.com/PyO3/pyo3/issues/759>
///
/// # Errors
///
/// May return Err Result if can't build module or change modules.
pub fn add_module(
    py: Python<'_>,
    parent_mod: &PyModule,
    child_mod_name: &'static str,
    child_mod_builder: impl FnOnce(Python<'_>, &PyModule) -> PyResult<()>,
) -> PyResult<()> {
    let sub_module = PyModule::new(py, child_mod_name)?;
    child_mod_builder(py, sub_module)?;
    parent_mod.add_submodule(sub_module)?;
    py.import("sys")?.getattr("modules")?.set_item(
        format!("{}.{}", parent_mod.name()?, child_mod_name),
        sub_module,
    )?;
    Ok(())
}

/// Simple wrapper for pyo3 `pyo3_asyncio::tokio::future_into_py`.
///
/// It wraps incoming Future and return internal Result.
///
/// # Errors
///
/// May return Err Result if future acts incorrect.
pub fn rustengine_future<F, T>(py: Python<'_>, future: F) -> RustPSQLDriverPyResult<&PyAny>
where
    F: Future<Output = RustPSQLDriverPyResult<T>> + Send + 'static,
    T: IntoPy<PyObject>,
{
    let res = pyo3_asyncio::tokio::future_into_py(py, async { future.await.map_err(Into::into) })
        .map(Into::into)?;
    Ok(res)
}
