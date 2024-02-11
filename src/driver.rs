use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use pyo3::{pyclass, pymethods, types::PyString, Py, PyAny, PyObject, PyRef, PyRefMut, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::{types::ToSql, NoTls};

use crate::{
    common::rustengine_future,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO},
};

/// Transaction for internal use only.
///
/// It is not exposed to python.
pub struct RustTransaction {
    db_client: Arc<tokio::sync::RwLock<Object>>,
    is_started: Arc<tokio::sync::RwLock<bool>>,
    is_done: Arc<tokio::sync::RwLock<bool>>,
    rollback_savepoint: Arc<tokio::sync::RwLock<HashSet<String>>>,
}

impl RustTransaction {
    /// Execute querystring with parameters.
    ///
    /// Method doesn't acquire lock on any structure fields.
    /// It prepares and caches querystring in the inner Object object.
    ///
    /// Then execute the query.
    ///
    /// # Errors:
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done already
    /// 3) Can not create/retrieve prepared statement
    /// 4) Can not execute statement
    pub async fn inner_execute<'a>(
        &'a self,
        querystring: String,
        parameters: Vec<PythonDTO>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let db_client_guard = db_client_arc.read().await;
        let is_started_guard = is_started_arc.read().await;
        let is_done_guard = is_done_arc.read().await;

        if !*is_started_guard {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is not started, please call begin() on transaction".into(),
            ));
        }
        if *is_done_guard {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        }

        let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(parameters.len());
        for param in parameters.iter() {
            vec_parameters.push(param);
        }

        let statement: tokio_postgres::Statement =
            db_client_guard.prepare_cached(&querystring).await?;

        let result = db_client_guard
            .query(&statement, &vec_parameters.into_boxed_slice())
            .await?;

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Start the transaction.
    ///
    /// Execute `BEGIN` commands and mark transaction as `started`.
    ///
    /// # Errors:
    ///
    /// May return Err Result if:
    /// 1) Transaction is already started.
    /// 2) Transaction is done.
    /// 3) Cannot execute `BEGIN` command.
    pub async fn inner_begin<'a>(&'a self) -> RustPSQLDriverPyResult<()> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            is_started_guard.clone()
        };
        if started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already started".into(),
            ));
        }

        let done = {
            let is_done_guard = is_done_arc.read().await;
            is_done_guard.clone()
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        }

        let db_client_guard = db_client_arc.read().await;
        db_client_guard.batch_execute("BEGIN").await?;
        let mut is_started_write_guard = is_started_arc.write().await;
        *is_started_write_guard = true;

        Ok(())
    }

    /// Commit the transaction.
    ///
    /// Execute `COMMIT` command and mark transaction as `done`.
    ///
    /// # Errors:
    ///
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done
    /// 3) Cannot execute `COMMIT` command
    pub async fn inner_commit<'a>(&'a self) -> RustPSQLDriverPyResult<()> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            is_started_guard.clone()
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        }

        let done = {
            let is_done_guard = is_done_arc.read().await;
            is_done_guard.clone()
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        }

        let db_client_guard = db_client_arc.read().await;
        db_client_guard.batch_execute("COMMIT;").await?;
        let mut is_done_write_guard = is_done_arc.write().await;
        *is_done_write_guard = true;

        Ok(())
    }

    /// Create new SAVEPOINT.
    ///
    /// Execute SAVEPOINT <name of the savepoint> and
    /// add it to the transaction rollback_savepoint HashSet
    ///
    /// # Errors:
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done
    /// 3) Specified savepoint name is exists
    /// 4) Can not execute SAVEPOINT command
    pub async fn inner_savepoint<'a>(
        &'a self,
        savepoint_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            is_started_guard.clone()
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        }

        let done = {
            let is_done_guard = is_done_arc.read().await;
            is_done_guard.clone()
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        };

        let is_savepoint_name_exists = {
            let rollback_savepoint_read_guard = self.rollback_savepoint.read().await;
            rollback_savepoint_read_guard.contains(&savepoint_name)
        };

        if is_savepoint_name_exists {
            return Err(RustPSQLDriverError::DataBaseTransactionError(format!(
                "SAVEPOINT name {} is already taken by this transaction",
                savepoint_name
            )));
        }

        let db_client_guard = db_client_arc.read().await;
        db_client_guard
            .batch_execute(format!("SAVEPOINT {}", savepoint_name).as_str())
            .await?;
        let mut rollback_savepoint_guard = self.rollback_savepoint.write().await;
        rollback_savepoint_guard.insert(savepoint_name);
        Ok(())
    }

    /// Execute ROLLBACK command.
    ///
    /// Run ROLLBACK command and mark the transaction as done.
    ///
    /// # Errors:
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done
    /// 3) Can not execute ROLLBACK command
    pub async fn inner_rollback<'a>(&'a self) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            is_started_guard.clone()
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        };

        let done = {
            let is_done_guard = is_done_arc.read().await;
            is_done_guard.clone()
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        };

        let db_client_arc = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;
        db_client_guard.batch_execute("ROLLBACK").await?;
        let mut is_done_write_guard = is_done_arc.write().await;
        *is_done_write_guard = true;
        Ok(())
    }

    /// ROLLBACK to the specified savepoint
    ///
    /// Execute ROLLBACK TO SAVEPOINT <name of the savepoint>.
    ///
    /// # Errors:
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done
    /// 3) Specified savepoint name doesn't exist
    /// 4) Can not execute ROLLBACK TO SAVEPOINT command
    pub async fn inner_rollback_to<'a>(
        &'a self,
        rollback_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let started = {
            let is_started_guard = is_started_arc.read().await;
            is_started_guard.clone()
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        };

        let is_done_arc = self.is_done.clone();
        let done = {
            let is_done_guard = is_done_arc.read().await;
            is_done_guard.clone()
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        };

        let rollback_savepoint_arc = self.rollback_savepoint.clone();
        let is_rollback_exists = {
            let rollback_savepoint_guard = rollback_savepoint_arc.read().await;
            rollback_savepoint_guard.contains(&rollback_name)
        };
        if !is_rollback_exists {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Don't have rollback with this name".into(),
            ));
        }

        let db_client_arc = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;
        db_client_guard
            .batch_execute(format!("ROLLBACK TO SAVEPOINT {}", rollback_name).as_str())
            .await?;

        Ok(())
    }

    /// Execute RELEASE SAVEPOINT.
    ///
    /// Run RELEASE SAVEPOINT command.
    ///
    /// # Errors:
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done
    /// 3) Specified savepoint name doesn't exists
    /// 4) Can not execute RELEASE SAVEPOINT command
    pub async fn inner_release_savepoint<'a>(
        &'a self,
        rollback_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let started = {
            let is_started_guard = is_started_arc.read().await;
            is_started_guard.clone()
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        };

        let is_done_arc = self.is_done.clone();
        let done = {
            let is_done_guard = is_done_arc.read().await;
            is_done_guard.clone()
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        };

        let rollback_savepoint_arc = self.rollback_savepoint.clone();
        let is_rollback_exists = {
            let rollback_savepoint_guard = rollback_savepoint_arc.read().await;
            rollback_savepoint_guard.contains(&rollback_name)
        };
        if !is_rollback_exists {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Don't have rollback with this name".into(),
            ));
        }

        let db_client_arc = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;
        db_client_guard
            .batch_execute(format!("RELEASE SAVEPOINT {}", rollback_name).as_str())
            .await?;

        Ok(())
    }
}

#[pyclass()]
pub struct Transaction {
    transaction: Arc<tokio::sync::RwLock<RustTransaction>>,
}

#[pymethods]
impl Transaction {
    #[must_use]
    pub fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __anext__(&self, py: Python<'_>) -> RustPSQLDriverPyResult<Option<PyObject>> {
        let transaction_clone = self.transaction.clone();
        let future = rustengine_future(py, async move {
            Ok(Transaction {
                transaction: transaction_clone,
            })
        });
        Ok(Some(future?.into()))
    }

    pub fn __await__<'a>(
        slf: PyRefMut<'a, Self>,
        _py: Python,
    ) -> RustPSQLDriverPyResult<PyRefMut<'a, Self>> {
        Ok(slf)
    }

    fn __aenter__<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let transaction_arc = slf.transaction.clone();
        let transaction_arc2 = slf.transaction.clone();
        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_begin().await?;
            Ok(Transaction {
                transaction: transaction_arc2,
            })
        })
    }

    fn __aexit__<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        _exception_type: Py<PyAny>,
        _exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let transaction_arc = slf.transaction.clone();
        let transaction_arc2 = slf.transaction.clone();
        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_commit().await?;
            Ok(Transaction {
                transaction: transaction_arc2,
            })
        })
    }

    /// Execute querystring with parameters.
    ///
    /// It converts incoming parameters to rust readable
    /// and then execute the query with them.
    ///
    /// # Errors:
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            Ok(transaction_guard.inner_execute(querystring, params).await?)
        })
    }

    /// Start the transaction.
    ///
    /// # Errors:
    /// May return Err Result if cannot execute command.
    pub fn begin<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_begin().await?;

            Ok(())
        })
    }

    /// Commit the transaction.
    ///
    /// # Errors:
    /// May return Err Result if cannot execute command.
    pub fn commit<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_commit().await?;

            Ok(())
        })
    }

    /// Create new SAVEPOINT.
    ///
    /// # Errors:
    /// May return Err Result if cannot extract string
    /// or `inner_savepoint` returns
    pub fn savepoint<'a>(
        &'a self,
        py: Python<'a>,
        savepoint_name: &'a PyAny,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let py_string = {
            if savepoint_name.is_instance_of::<PyString>() {
                savepoint_name.extract::<String>()?
            } else {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "Can't convert your savepoint_name to String value".into(),
                ));
            }
        };

        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_savepoint(py_string).await?;

            Ok(())
        })
    }

    /// Rollback the whole transaction.
    ///
    /// # Errors:
    /// May return Err Result if `rollback` returns Error.
    pub fn rollback<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_rollback().await?;

            Ok(())
        })
    }

    /// Rollback to the specified savepoint.
    ///
    /// # Errors:
    /// May return Err Result if cannot extract string
    /// or`inner_rollback_to` returns Error.
    pub fn rollback_to<'a>(
        &'a self,
        py: Python<'a>,
        savepoint_name: &'a PyAny,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let py_string = {
            if savepoint_name.is_instance_of::<PyString>() {
                savepoint_name.extract::<String>()?
            } else {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "Can't convert your savepoint_name to String value".into(),
                ));
            }
        };

        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_rollback_to(py_string).await?;

            Ok(())
        })
    }

    /// Rollback to the specified savepoint.
    ///
    /// # Errors:
    /// May return Err Result if cannot extract string
    /// or`inner_rollback_to` returns Error.
    pub fn release_savepoint<'a>(
        &'a self,
        py: Python<'a>,
        savepoint_name: &'a PyAny,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let py_string = {
            if savepoint_name.is_instance_of::<PyString>() {
                savepoint_name.extract::<String>()?
            } else {
                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "Can't convert your savepoint_name to String value".into(),
                ));
            }
        };

        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_release_savepoint(py_string).await?;

            Ok(())
        })
    }
}

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
    pub async fn inner_transaction<'a>(&'a self) -> RustPSQLDriverPyResult<Transaction> {
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
    pub fn transaction<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&'a PyAny> {
        let psql_pool_arc = self.rust_psql_pool.clone();

        rustengine_future(py, async move {
            let psql_pool_guard = psql_pool_arc.write().await;

            let transaction = psql_pool_guard.inner_transaction().await?;

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
