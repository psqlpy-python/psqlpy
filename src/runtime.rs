use futures_util::Future;
use pyo3::{IntoPy, Py, PyAny, PyObject, Python};

use crate::exceptions::rust_errors::RustPSQLDriverPyResult;

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
pub fn rustdriver_future<F, T>(py: Python<'_>, future: F) -> RustPSQLDriverPyResult<Py<PyAny>>
where
    F: Future<Output = RustPSQLDriverPyResult<T>> + Send + 'static,
    T: IntoPy<PyObject>,
{
    let res = pyo3_asyncio::tokio::future_into_py(py, async { future.await.map_err(Into::into) })
        .map(Into::into)?;
    Ok(res)
}
