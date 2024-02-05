use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use pyo3::{pyclass, pymethods, IntoPy, PyAny, PyObject, Python};
use std::{future::Future, sync::Arc, vec};
use tokio_postgres::{types::ToSql, NoTls};

use crate::{
    exceptions::rust_errors::{RustEngineError, RustEnginePyResult},
    query_result::RustEnginePyQueryResult,
    value_converter::{convert_parameters, PythonType},
};

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
pub struct RustEngineTransaction {
    db_client: Arc<tokio::sync::RwLock<Object>>,
    is_started: Arc<tokio::sync::RwLock<bool>>,
}

#[pymethods]
impl RustEngineTransaction {
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustEnginePyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();

        let mut params: Vec<PythonType> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let started = {
                let is_started_guard = is_started_arc.read().await;
                is_started_guard.clone()
            };

            if !started {
                let mut is_started_write_guard = is_started_arc.write().await;
                println!("Called BEGIN!");
                db_client_guard.batch_execute("BEGIN").await?;
                *is_started_write_guard = true;
            };

            let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(params.len());
            for param in params.iter() {
                vec_parameters.push(param);
            }

            let result = db_client_guard
                .query(&querystring, &vec_parameters.into_boxed_slice())
                .await?;

            Ok(RustEnginePyQueryResult::new(result))
        })
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
    pub async fn inner_execute<'a>(
        &'a self,
        querystring: String,
        parameters: Vec<PythonType>,
    ) -> RustEnginePyResult<RustEnginePyQueryResult> {
        let db_pool_arc = self.db_pool.clone();

        let db_pool_guard = db_pool_arc.read().await;

        let db_pool_manager = db_pool_guard
            .as_ref()
            .ok_or(RustEngineError::DatabasePoolError(
                "Database pool is not initialized".into(),
            ))?
            .get()
            .await?;

        let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(parameters.len());
        for param in parameters.iter() {
            vec_parameters.push(param);
        }

        let result = db_pool_manager
            .query(
                &db_pool_manager.prepare_cached(&querystring).await?,
                &vec_parameters.into_boxed_slice(),
            )
            .await?;
        Ok(RustEnginePyQueryResult::new(result))
    }

    pub async fn inner_transaction<'a>(&'a self) -> RustEnginePyResult<RustEngineTransaction> {
        let db_pool_arc = self.db_pool.clone();
        let db_pool_guard = db_pool_arc.read().await;

        let db_pool_manager = db_pool_guard
            .as_ref()
            .ok_or(RustEngineError::DatabasePoolError(
                "Database pool is not initialized".into(),
            ))?
            .get()
            .await?;

        Ok(RustEngineTransaction {
            db_client: Arc::new(tokio::sync::RwLock::new(db_pool_manager)),
            is_started: Arc::new(tokio::sync::RwLock::new(false)),
        })
    }

    pub async fn inner_startup<'a>(&'a self) -> RustEnginePyResult<()> {
        let db_pool_arc = self.db_pool.clone();
        let password = self.password.clone();
        let username = self.username.clone();
        let db_host = self.host.clone();
        let db_port = self.port;
        let db_name = self.db_name.clone();

        let mut db_pool_guard = db_pool_arc.write().await;
        if db_pool_guard.is_some() {
            return Err(RustEngineError::DatabasePoolError(
                "Database pool is already initialized".into(),
            ));
        }

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

        *db_pool_guard = Some(Pool::builder(mgr).max_size(1).build()?);
        Ok(())
    }
}

impl RustEngine {
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
}

#[pyclass()]
pub struct PyRustEngine {
    engine: Arc<tokio::sync::RwLock<Option<RustEngine>>>,
}

#[pymethods]
impl PyRustEngine {
    #[new]
    pub fn new(
        username: Option<String>,
        password: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        db_name: Option<String>,
    ) -> Self {
        PyRustEngine {
            engine: Arc::new(tokio::sync::RwLock::new(Some(RustEngine {
                username,
                password,
                host,
                port,
                db_name,
                db_pool: Arc::new(tokio::sync::RwLock::new(None)),
            }))),
        }
    }

    pub fn startup<'a>(&'a self, py: Python<'a>) -> RustEnginePyResult<&'a PyAny> {
        let db_engine_arc = self.engine.clone();
        rustengine_future(py, async move {
            let mut db_pool_guard = db_engine_arc.write().await;
            db_pool_guard.as_mut().unwrap().inner_startup().await
        })
    }

    pub fn transaction<'a>(&'a self, py: Python<'a>) -> RustEnginePyResult<&'a PyAny> {
        let engine_arc = self.engine.clone();

        rustengine_future(py, async move {
            let mut engine_guard = engine_arc.write().await;

            let transaction = engine_guard
                .as_mut()
                .unwrap()
                .inner_transaction()
                .await
                .unwrap();

            Ok(transaction)
        })
    }

    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustEnginePyResult<&'a PyAny> {
        let engine_arc = self.engine.clone();
        let mut params: Vec<PythonType> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let engine_guard = engine_arc.read().await;

            engine_guard
                .as_ref()
                .unwrap()
                .inner_execute(querystring, params)
                .await
        })
    }
}
