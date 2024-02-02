use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use pyo3::{
    pyclass, pymethods,
    types::{PyList, PyTuple},
    IntoPy, PyAny, PyObject, Python,
};
use std::{future::Future, sync::Arc, vec};
use tokio_postgres::{types::ToSql, NoTls};

use pyo3::create_exception;

use thiserror::Error;

use crate::{
    query_result::RustEnginePyQueryResult,
    value_converter::{py_to_rust, PythonType},
};

create_exception!(
    rustengine.exceptions,
    RustEnginePyBaseError,
    pyo3::exceptions::PyException
);

#[derive(Error, Debug)]
pub enum RustEngineError {
    #[error("Database pool error: {0}.")]
    DatabasePoolError(String),
    #[error("Can't convert value from engine to python type: {0}")]
    ValueConversionError(String),

    #[error("Python exception: {0}.")]
    PyError(#[from] pyo3::PyErr),
    #[error("Database engine exception: {0}.")]
    DBEngineError(#[from] tokio_postgres::Error),
    #[error("Database engine pool exception: {0}")]
    DBEnginePoolError(#[from] deadpool_postgres::PoolError),
}

pub type RustEnginePyResult<T> = Result<T, RustEngineError>;

impl From<RustEngineError> for pyo3::PyErr {
    fn from(error: RustEngineError) -> Self {
        let error_desc = error.to_string();
        match error {
            RustEngineError::PyError(err) => err,
            RustEngineError::DBEngineError(_) => RustEnginePyBaseError::new_err((error_desc,)),
            RustEngineError::ValueConversionError(_) => {
                RustEnginePyBaseError::new_err((error_desc,))
            }
            RustEngineError::DatabasePoolError(_) => RustEnginePyBaseError::new_err((error_desc,)),
            RustEngineError::DBEnginePoolError(_) => RustEnginePyBaseError::new_err((error_desc,)),
        }
    }
}

pub fn rustengine_future<F, T>(py: Python<'_>, future: F) -> RustEnginePyResult<&PyAny>
where
    F: Future<Output = RustEnginePyResult<T>> + Send + 'static,
    T: IntoPy<PyObject>,
{
    let res = pyo3_asyncio::tokio::future_into_py(py, async { future.await.map_err(Into::into) })
        .map(Into::into)?;
    Ok(res)
}

#[pyclass()]
pub struct RustEngine {
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    db_name: Option<String>,
    db_pool: Arc<tokio::sync::RwLock<Option<Pool>>>,
}

impl RustEngine {
    pub fn inner_execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Vec<PythonType>,
    ) -> RustEnginePyResult<&'a PyAny> {
        let db_pool_arc = self.db_pool.clone();

        rustengine_future(py, async move {
            let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(parameters.len());
            for param in parameters.iter() {
                vec_parameters.push(param);
            }

            let db_pool_guard = db_pool_arc.read().await;

            let db_pool_manager = db_pool_guard
                .as_ref()
                .ok_or(RustEngineError::DatabasePoolError(
                    "Database pool is not initialized".into(),
                ))?
                .get()
                .await?;

            let result = db_pool_manager
                .query(&querystring, &vec_parameters.into_boxed_slice())
                .await?;
            Ok(RustEnginePyQueryResult::new(result))
        })
        .map_err(Into::into)
    }
}

#[pymethods]
impl RustEngine {
    #[new]
    pub fn new(
        username: Option<String>,
        password: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        db_name: Option<String>,
    ) -> Self {
        RustEngine {
            username,
            password,
            host,
            port,
            db_name,
            db_pool: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    #[pyo3()]
    pub fn startup<'a>(&'a self, py: Python<'a>) -> RustEnginePyResult<&'a PyAny> {
        let db_pool_arc = self.db_pool.clone();
        let password = self.password.clone();
        let username = self.username.clone();
        let db_host = self.host.clone();
        let db_port = self.port;
        let db_name = self.db_name.clone();

        rustengine_future(py, async move {
            let mut session_guard = db_pool_arc.write().await;

            let mut pg_config = tokio_postgres::Config::new();

            if let (Some(password), Some(username)) = (password, username) {
                pg_config.password(&password);
                pg_config.user(&username);
            }
            if let Some(db_host) = db_host {
                pg_config.host(&db_host);
            }

            if let Some(db_port) = db_port {
                pg_config.port(db_port);
            }

            if let Some(db_name) = db_name {
                pg_config.dbname(&db_name);
            }

            let mgr_config = ManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            };
            let mgr = Manager::from_config(pg_config, NoTls, mgr_config);

            *session_guard = Some(Pool::builder(mgr).max_size(16).build().unwrap());
            Ok(())
        })
    }

    #[pyo3()]
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustEnginePyResult<&'a PyAny> {
        let mut result_vec: Vec<PythonType> = vec![];

        if parameters.unwrap().is_instance_of::<PyList>()
            || parameters.unwrap().is_instance_of::<PyTuple>()
        {
            let params = parameters.unwrap().extract::<Vec<&PyAny>>();
            for parameter in params?.iter() {
                result_vec.push(py_to_rust(parameter).unwrap());
            }
        }
        self.inner_execute(py, querystring, result_vec)
    }
}
