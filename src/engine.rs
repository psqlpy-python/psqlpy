use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use pyo3::{
    pyclass, pymethods,
    types::{IntoPyDict, PyDict, PyList, PyString},
    IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use serde_json::{Map, Value};
use std::{
    borrow::BorrowMut, collections::HashMap, future::Future, hash::BuildHasherDefault,
    str::FromStr, sync::Arc,
};
use tokio_postgres::{
    types::{FromSql, ToSql, Type},
    Column, NoTls, Row, ToStatement,
};

use thiserror::Error;

use crate::value_converter::postgres_to_py;

#[derive(Error, Debug)]
pub enum RustEngineError {
    #[error("Python exception: {0}.")]
    PyError(#[from] pyo3::PyErr),
}

pub type RustEnginePyResult<T> = Result<T, RustEngineError>;

impl From<RustEngineError> for pyo3::PyErr {
    fn from(error: RustEngineError) -> Self {
        match error {
            RustEngineError::PyError(err) => err,
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

#[pyclass(name = "QueryResult")]
pub struct RustEnginePyQueryResult {
    inner: Vec<Row>,
}

#[pymethods]
impl RustEnginePyQueryResult {
    pub fn result<'a>(&self, py: Python<'a>) -> Result<Py<PyAny>, RustEngineError> {
        let mut result: Vec<&PyDict> = vec![];
        for row in &self.inner {
            let python_dict = PyDict::new(py);
            for (column_idx, column) in row.columns().iter().enumerate() {
                let python_type = postgres_to_py(py, row, column, column_idx)?;
                python_dict.set_item(column.name().to_object(py), python_type)?;
            }
            result.push(python_dict);
        }

        Ok(result.to_object(py))
        // let a = &self.inner[0];
        // let json_string = postgres_row_to_json_value(a).unwrap();
        // Ok(PyString::new(py, &json_string).as_ref())
    }
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
    // pub async fn inner_execute<'a, QS>(
    pub fn inner_execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        // parameters: Option<&'static [&(dyn ToSql + Sync)]>,
    ) -> RustEnginePyResult<&'a PyAny> {
        let db_pool_arc = self.db_pool.clone();

        rustengine_future(py, async move {
            let db_pool_guard = db_pool_arc.read().await;

            let result = db_pool_guard
                .as_ref()
                .unwrap()
                .get()
                .await
                .unwrap()
                // .query(querystring, parameters.unwrap_or_default())
                .query(&querystring, &[])
                .await
                .unwrap();
            println!("You guessed: {:?}", result);
            Ok(RustEnginePyQueryResult { inner: result })
        })
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
        // parameters: Option<&'a PyAny>,
    ) -> RustEnginePyResult<&'a PyAny> {
        self.inner_execute(py, querystring)
    }
}
