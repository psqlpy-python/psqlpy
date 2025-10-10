use futures_util::Future;
use pyo3::{IntoPyObject, Py, PyAny, Python};

use crate::exceptions::rust_errors::PSQLPyResult;

#[allow(clippy::missing_panics_doc)]
#[allow(clippy::module_name_repetitions)]
pub fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

/// Simple wrapper for pyo3 `pyo3_asyncio::tokio::future_into_py`.
///
/// It wraps incoming Future and return internal Result.
///
/// # Errors
///
/// May return Err Result if future acts incorrect.
pub fn rustdriver_future<F, T>(py: Python<'_>, future: F) -> PSQLPyResult<Py<PyAny>>
where
    F: Future<Output = PSQLPyResult<T>> + Send + 'static,
    T: for<'py> IntoPyObject<'py>,
{
    let res =
        pyo3_async_runtimes::tokio::future_into_py(py, async { future.await.map_err(Into::into) })
            .map(Into::into)?;
    Ok(res)
}
