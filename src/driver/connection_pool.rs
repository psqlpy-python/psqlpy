use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use pyo3::{pyclass, pymethods, PyAny, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::{types::ToSql, NoTls};

use crate::{
    common::rustengine_future,
    driver::transaction::{RustTransaction, Transaction},
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO},
};

use super::transaction_options::IsolationLevel;

/// PSQLPool for internal use only.
///
/// It is not exposed to python.
pub struct RustPSQLPool {
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    db_name: Option<String>,
    max_db_pool_size: Option<usize>,
    db_pool: Arc<tokio::sync::RwLock<Option<Pool>>>,
}

impl RustPSQLPool {
    /// Create new `RustPSQLPool`.
    pub fn new(
        username: Option<String>,
        password: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        db_name: Option<String>,
        max_db_pool_size: Option<usize>,
    ) -> Self {
        RustPSQLPool {
            username,
            password,
            host,
            port,
            db_name,
            max_db_pool_size,
            db_pool: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
}

impl RustPSQLPool {
    /// Execute querystring with parameters.
    ///
    /// Prepare statement and cache it, then execute.
    ///
    /// # Errors:
    /// May return Err Result if cannot retrieve new connection
    /// or prepare statement or execute statement.
    pub async fn inner_execute<'a>(
        &'a self,
        querystring: String,
        parameters: Vec<PythonDTO>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_pool_arc = self.db_pool.clone();

        let db_pool_guard = db_pool_arc.read().await;

        let db_pool_manager = db_pool_guard
            .as_ref()
            .ok_or(RustPSQLDriverError::DatabasePoolError(
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
        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Create new inner transaction and return it.
    ///
    /// # Errors:
    /// May return Err Result if cannot retrieve connection from the pool.
    pub async fn inner_transaction<'a>(
        &'a self,
        isolation_level: Option<IsolationLevel>,
    ) -> RustPSQLDriverPyResult<Transaction> {
        let db_pool_arc = self.db_pool.clone();
        let db_pool_guard = db_pool_arc.read().await;

        let db_pool_manager = db_pool_guard
            .as_ref()
            .ok_or(RustPSQLDriverError::DatabasePoolError(
                "Database pool is not initialized".into(),
            ))?
            .get()
            .await?;

        let inner_transaction = RustTransaction {
            db_client: Arc::new(tokio::sync::RwLock::new(db_pool_manager)),
            is_started: Arc::new(tokio::sync::RwLock::new(false)),
            is_done: Arc::new(tokio::sync::RwLock::new(false)),
            rollback_savepoint: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
            isolation_level: isolation_level,
        };

        Ok(Transaction {
            transaction: Arc::new(tokio::sync::RwLock::new(inner_transaction)),
        })
    }

    /// Create new Database pool.
    ///
    /// # Errors:
    /// May return Err Result if Database pool is already initialized,
    /// max_db_pool_size is less than 2 or it's impossible to build db pool.
    pub async fn inner_startup<'a>(&'a self) -> RustPSQLDriverPyResult<()> {
        let db_pool_arc = self.db_pool.clone();
        let password = self.password.clone();
        let username = self.username.clone();
        let db_host = self.host.clone();
        let db_port = self.port;
        let db_name = self.db_name.clone();
        let max_db_pool_size = self.max_db_pool_size.clone();

        let mut db_pool_guard = db_pool_arc.write().await;
        if db_pool_guard.is_some() {
            return Err(RustPSQLDriverError::DatabasePoolError(
                "Database pool is already initialized".into(),
            ));
        }

        if let Some(max_db_pool_size) = max_db_pool_size {
            if max_db_pool_size < 2 {
                return Err(RustPSQLDriverError::DataBasePoolConfigurationError(
                    "Maximum database pool size must be more than 1".into(),
                ));
            }
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

        let mut db_pool_builder = Pool::builder(mgr);
        if let Some(max_db_pool_size) = max_db_pool_size {
            db_pool_builder = db_pool_builder.max_size(max_db_pool_size);
        }

        *db_pool_guard = Some(db_pool_builder.build()?);
        Ok(())
    }
}

#[pyclass()]
pub struct PSQLPool {
    rust_psql_pool: Arc<tokio::sync::RwLock<RustPSQLPool>>,
}

#[pymethods]
impl PSQLPool {
    #[new]
    pub fn new(
        username: Option<String>,
        password: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        db_name: Option<String>,
        max_db_pool_size: Option<usize>,
    ) -> Self {
        PSQLPool {
            rust_psql_pool: Arc::new(tokio::sync::RwLock::new(RustPSQLPool {
                username,
                password,
                host,
                port,
                db_name,
                max_db_pool_size,
                db_pool: Arc::new(tokio::sync::RwLock::new(None)),
            })),
        }
    }

    /// Startup Database Pool.
    ///
    /// # Errors:
    /// May return Err Result if `inner_startup` returns error.
    pub fn startup<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&'a PyAny> {
        let psql_pool_arc = self.rust_psql_pool.clone();
        rustengine_future(py, async move {
            let db_pool_guard = psql_pool_arc.write().await;
            db_pool_guard.inner_startup().await?;
            Ok(())
        })
    }

    /// Return python transaction.
    ///
    /// # Errors:
    /// May return Err Result if `inner_transaction` returns error.
    pub fn transaction<'a>(
        &'a self,
        py: Python<'a>,
        isolation_level: Option<IsolationLevel>,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let psql_pool_arc = self.rust_psql_pool.clone();

        rustengine_future(py, async move {
            let psql_pool_guard = psql_pool_arc.write().await;

            let transaction = psql_pool_guard.inner_transaction(isolation_level).await?;

            Ok(transaction)
        })
    }

    /// Execute querystring with parameters.
    ///
    /// # Errors:
    /// May return Err Result if cannot convert parameters
    /// or `inner_execute` returns Err.
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let engine_arc = self.rust_psql_pool.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let engine_guard = engine_arc.read().await;

            Ok(engine_guard.inner_execute(querystring, params).await?)
        })
    }
}
