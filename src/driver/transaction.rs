use deadpool_postgres::Object;
use pyo3::{pyclass, pymethods, types::PyString, Py, PyAny, PyObject, PyRef, PyRefMut, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::types::ToSql;

use crate::{
    common::rustengine_future,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO},
};

use super::{
    cursor::Cursor,
    transaction_options::{IsolationLevel, ReadVariant},
};

/// Transaction for internal use only.
///
/// It is not exposed to python.
pub struct RustTransaction {
    db_client: Arc<tokio::sync::RwLock<Object>>,
    is_started: Arc<tokio::sync::RwLock<bool>>,
    is_done: Arc<tokio::sync::RwLock<bool>>,
    rollback_savepoint: Arc<tokio::sync::RwLock<HashSet<String>>>,

    isolation_level: Option<IsolationLevel>,
    read_variant: Option<ReadVariant>,
    cursor_num: usize,
}

impl RustTransaction {
    pub fn new(
        db_client: Arc<tokio::sync::RwLock<Object>>,
        is_started: Arc<tokio::sync::RwLock<bool>>,
        is_done: Arc<tokio::sync::RwLock<bool>>,
        rollback_savepoint: Arc<tokio::sync::RwLock<HashSet<String>>>,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        cursor_num: usize,
    ) -> Self {
        Self {
            db_client,
            is_started,
            is_done,
            rollback_savepoint,
            isolation_level,
            read_variant,
            cursor_num,
        }
    }

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
    pub async fn inner_execute(
        &self,
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

        let statement = db_client_guard.prepare_cached(&querystring).await?;

        let result = db_client_guard
            .query(&statement, &vec_parameters.into_boxed_slice())
            .await?;

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Start transaction with isolation level if specified
    ///
    /// # Errors:
    /// May return Err Result if cannot execute querystring.
    pub async fn start_transaction(&self) -> RustPSQLDriverPyResult<()> {
        let mut querystring = "START TRANSACTION".to_string();

        if let Some(level) = self.isolation_level {
            let level = &level.to_str_level();
            querystring.push_str(format!(" ISOLATION LEVEL {level}").as_str());
        };

        if let Some(read_var) = self.read_variant {
            querystring.push_str(format!(" {}", &read_var.to_str_option()).as_str())
        }

        let db_client_arc = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;

        db_client_guard.batch_execute(&querystring).await?;

        Ok(())
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
    pub async fn inner_begin(&self) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            *is_started_guard
        };
        if started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already started".into(),
            ));
        }

        let done = {
            let is_done_guard = is_done_arc.read().await;
            *is_done_guard
        };
        if done {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Transaction is already committed or rolled back".into(),
            ));
        }

        self.start_transaction().await?;
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
    pub async fn inner_commit(&self) -> RustPSQLDriverPyResult<()> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            *is_started_guard
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        }

        let done = {
            let is_done_guard = is_done_arc.read().await;
            *is_done_guard
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
    pub async fn inner_savepoint(&self, savepoint_name: String) -> RustPSQLDriverPyResult<()> {
        let db_client_arc = self.db_client.clone();
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            *is_started_guard
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        }

        let done = {
            let is_done_guard = is_done_arc.read().await;
            *is_done_guard
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
    pub async fn inner_rollback(&self) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let is_done_arc = self.is_done.clone();

        let started = {
            let is_started_guard = is_started_arc.read().await;
            *is_started_guard
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        };

        let done = {
            let is_done_guard = is_done_arc.read().await;
            *is_done_guard
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
    pub async fn inner_rollback_to(&self, rollback_name: String) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let started = {
            let is_started_guard = is_started_arc.read().await;
            *is_started_guard
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        };

        let is_done_arc = self.is_done.clone();
        let done = {
            let is_done_guard = is_done_arc.read().await;
            *is_done_guard
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
    pub async fn inner_release_savepoint(
        &self,
        rollback_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let is_started_arc = self.is_started.clone();
        let started = {
            let is_started_guard = is_started_arc.read().await;
            *is_started_guard
        };
        if !started {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Can not commit not started transaction".into(),
            ));
        };

        let is_done_arc = self.is_done.clone();
        let done = {
            let is_done_guard = is_done_arc.read().await;
            *is_done_guard
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

    pub async fn inner_cursor(
        &mut self,
        querystring: String,
        parameters: Vec<PythonDTO>,
        fetch_number: usize,
        scroll: Option<bool>,
    ) -> RustPSQLDriverPyResult<Cursor> {
        let db_client_arc = self.db_client.clone();
        let db_client_arc2 = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;

        let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(parameters.len());
        for param in parameters.iter() {
            vec_parameters.push(param);
        }

        let mut cursor_init_query = "DECLARE".to_string();
        cursor_init_query.push_str(format!(" cur{}", self.cursor_num).as_str());

        if let Some(scroll) = scroll {
            if scroll {
                cursor_init_query.push_str(" SCROLL");
            } else {
                cursor_init_query.push_str(" NO SCROLL");
            }
        }

        cursor_init_query.push_str(format!(" CURSOR FOR {querystring}").as_str());

        let cursor_name = format!("cur{}", self.cursor_num);
        db_client_guard
            .execute(&cursor_init_query, &vec_parameters.into_boxed_slice())
            .await?;

        self.cursor_num += 1;

        Ok(Cursor::new(db_client_arc2, cursor_name, fetch_number))
    }
}

#[pyclass()]
pub struct Transaction {
    pub transaction: Arc<tokio::sync::RwLock<RustTransaction>>,
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
            transaction_guard.inner_execute(querystring, params).await
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

    pub fn cursor<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
        fetch_number: Option<usize>,
        scroll: Option<bool>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let mut transaction_guard = transaction_arc.write().await;
            transaction_guard
                .inner_cursor(querystring, params, fetch_number.unwrap_or(10), scroll)
                .await
        })
    }
}
