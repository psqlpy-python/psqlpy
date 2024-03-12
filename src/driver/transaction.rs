use super::{
    cursor::Cursor,
    transaction_options::{IsolationLevel, ReadVariant},
};
use crate::{
    common::rustengine_future,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, postgres_to_py, PythonDTO},
};
use deadpool_postgres::Object;
use futures_util::future;
use pyo3::{
    pyclass, pymethods,
    types::{PyList, PyString, PyTuple},
    Py, PyAny, PyErr, PyObject, PyRef, PyRefMut, Python,
};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::types::ToSql;

/// Transaction for internal use only.
///
/// It is not exposed to python.
#[allow(clippy::module_name_repetitions)]
pub struct RustTransaction {
    db_client: Arc<tokio::sync::RwLock<Object>>,
    is_started: Arc<tokio::sync::RwLock<bool>>,
    is_done: Arc<tokio::sync::RwLock<bool>>,
    rollback_savepoint: Arc<tokio::sync::RwLock<HashSet<String>>>,

    isolation_level: Option<IsolationLevel>,
    read_variant: Option<ReadVariant>,
    deferable: Option<bool>,
    cursor_num: usize,
}

impl RustTransaction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db_client: Arc<tokio::sync::RwLock<Object>>,
        is_started: Arc<tokio::sync::RwLock<bool>>,
        is_done: Arc<tokio::sync::RwLock<bool>>,
        rollback_savepoint: Arc<tokio::sync::RwLock<HashSet<String>>>,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferable: Option<bool>,
        cursor_num: usize,
    ) -> Self {
        Self {
            db_client,
            is_started,
            is_done,
            rollback_savepoint,
            isolation_level,
            read_variant,
            deferable,
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
    /// # Errors
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
        for param in &parameters {
            vec_parameters.push(param);
        }

        let statement = db_client_guard.prepare_cached(&querystring).await?;

        let result = db_client_guard
            .query(&statement, &vec_parameters.into_boxed_slice())
            .await?;

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Execute querystring with many parameters.
    ///
    /// Method doesn't acquire lock on any structure fields.
    /// It prepares and caches querystring in the inner Object object.
    ///
    /// Then execute the query.
    ///
    /// # Errors
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done already
    /// 3) Can not create/retrieve prepared statement
    /// 4) Can not execute statement
    pub async fn inner_execute_many(
        &self,
        querystring: String,
        parameters: Vec<Vec<PythonDTO>>,
    ) -> RustPSQLDriverPyResult<()> {
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
        if parameters.is_empty() {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "No parameters passed to execute_many".into(),
            ));
        }
        for single_parameters in parameters {
            let statement = db_client_guard.prepare_cached(&querystring).await?;
            db_client_guard
                .query(
                    &statement,
                    &single_parameters
                        .iter()
                        .map(|p| p as &(dyn ToSql + Sync))
                        .collect::<Vec<_>>(),
                )
                .await?;
        }

        Ok(())
    }

    /// Fetch exaclty single row from query.
    ///
    /// Method doesn't acquire lock on any structure fields.
    /// It prepares and caches querystring in the inner Object object.
    ///
    /// Then execute the query.
    ///
    /// # Errors
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done already
    /// 3) Can not create/retrieve prepared statement
    /// 4) Can not execute statement
    /// 5) Query returns more than one row
    pub async fn inner_fetch_row(
        &self,
        querystring: String,
        parameters: Vec<PythonDTO>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
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
        for param in &parameters {
            vec_parameters.push(param);
        }

        let statement = db_client_guard.prepare_cached(&querystring).await?;

        let result = db_client_guard
            .query_one(&statement, &vec_parameters.into_boxed_slice())
            .await?;

        Ok(PSQLDriverSinglePyQueryResult::new(result))
    }

    /// Run many queries as pipeline.
    ///
    /// It can boost up querying speed.
    ///
    /// # Errors
    ///
    /// May return Err Result if can't join futures or cannot execute
    /// any of queries.
    pub async fn inner_pipeline(
        &self,
        queries: Vec<(String, Vec<PythonDTO>)>,
    ) -> RustPSQLDriverPyResult<Vec<PSQLDriverPyQueryResult>> {
        let mut futures = vec![];
        for (querystring, params) in queries {
            let execute_future = self.inner_execute(querystring, params);
            futures.push(execute_future);
        }

        let b = future::try_join_all(futures).await?;
        Ok(b)
    }

    /// Start transaction
    /// Set up isolation level if specified
    /// Set up deferable if specified
    ///
    /// # Errors
    /// May return Err Result if cannot execute querystring.
    pub async fn start_transaction(&self) -> RustPSQLDriverPyResult<()> {
        let mut querystring = "START TRANSACTION".to_string();

        if let Some(level) = self.isolation_level {
            let level = &level.to_str_level();
            querystring.push_str(format!(" ISOLATION LEVEL {level}").as_str());
        };

        querystring.push_str(match self.read_variant {
            Some(ReadVariant::ReadOnly) => " READ ONLY",
            Some(ReadVariant::ReadWrite) => " READ WRITE",
            None => "",
        });

        querystring.push_str(match self.deferable {
            Some(true) => " DEFERRABLE",
            Some(false) => " NOT DEFERRABLE",
            None => "",
        });

        let db_client_arc = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;

        db_client_guard.batch_execute(&querystring).await?;

        Ok(())
    }

    /// Start the transaction.
    ///
    /// Execute `BEGIN` commands and mark transaction as `started`.
    ///
    /// # Errors
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
    /// # Errors
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
    /// add it to the transaction `rollback_savepoint` `HashSet`
    ///
    /// # Errors
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
                "SAVEPOINT name {savepoint_name} is already taken by this transaction",
            )));
        }

        let db_client_guard = db_client_arc.read().await;
        db_client_guard
            .batch_execute(format!("SAVEPOINT {savepoint_name}").as_str())
            .await?;
        let mut rollback_savepoint_guard = self.rollback_savepoint.write().await;
        rollback_savepoint_guard.insert(savepoint_name);
        Ok(())
    }

    /// Execute ROLLBACK command.
    ///
    /// Run ROLLBACK command and mark the transaction as done.
    ///
    /// # Errors
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
    /// # Errors
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
            .batch_execute(format!("ROLLBACK TO SAVEPOINT {rollback_name}").as_str())
            .await?;

        Ok(())
    }

    /// Execute RELEASE SAVEPOINT.
    ///
    /// Run RELEASE SAVEPOINT command.
    ///
    /// # Errors
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

        let mut rollback_savepoint_guard = self.rollback_savepoint.write().await;
        let is_rollback_exists = rollback_savepoint_guard.remove(&rollback_name);

        if !is_rollback_exists {
            return Err(RustPSQLDriverError::DataBaseTransactionError(
                "Don't have rollback with this name".into(),
            ));
        }

        let db_client_arc = self.db_client.clone();
        let db_client_guard = db_client_arc.read().await;
        db_client_guard
            .batch_execute(format!("RELEASE SAVEPOINT {rollback_name}").as_str())
            .await?;

        Ok(())
    }

    /// Create new cursor, init it and return new Cursor struct.
    ///
    /// Execute DECLARE statement with cursor name and querystring.
    ///
    /// # Errors
    /// May return Err Result if can't execute query.
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
        for param in &parameters {
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

    /// Return new instance of transaction.
    ///
    /// It's necessary because python requires it.
    ///
    /// # Errors
    /// May return Err Result if future returns error.
    pub fn __anext__(&self, py: Python<'_>) -> RustPSQLDriverPyResult<Option<PyObject>> {
        let transaction_clone = self.transaction.clone();
        let future = rustengine_future(py, async move {
            Ok(Transaction {
                transaction: transaction_clone,
            })
        });
        Ok(Some(future?.into()))
    }

    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn __await__<'a>(
        slf: PyRefMut<'a, Self>,
        _py: Python,
    ) -> RustPSQLDriverPyResult<PyRefMut<'a, Self>> {
        Ok(slf)
    }

    #[allow(clippy::needless_pass_by_value)]
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

    #[allow(clippy::needless_pass_by_value)]
    fn __aexit__<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        _exception_type: Py<PyAny>,
        exception: &PyAny,
        _traceback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let transaction_arc = slf.transaction.clone();
        let transaction_arc2 = slf.transaction.clone();
        let is_no_exc = exception.is_none();
        let py_err = PyErr::from_value(exception);

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            if is_no_exc {
                transaction_guard.inner_commit().await?;
                Ok(Transaction {
                    transaction: transaction_arc2,
                })
            } else {
                transaction_guard.inner_rollback().await?;
                Err(RustPSQLDriverError::PyError(py_err))
            }
        })
    }

    /// Execute querystring with parameters.
    ///
    /// It converts incoming parameters to rust readable
    /// and then execute the query with them.
    ///
    /// # Errors
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
            params = convert_parameters(parameters)?;
        }

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_execute(querystring, params).await
        })
    }

    /// Execute querystring with parameters.
    ///
    /// It converts incoming parameters to rust readable
    /// and then execute the query with them.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    pub fn execute_many<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyList>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();
        let mut params: Vec<Vec<PythonDTO>> = vec![];
        if let Some(parameters) = parameters {
            for single_parameters in parameters {
                params.push(convert_parameters(single_parameters)?);
            }
        }

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard
                .inner_execute_many(querystring, params)
                .await
        })
    }
    /// Execute querystring with parameters and return first row.
    ///
    /// It converts incoming parameters to rust readable,
    /// executes query with them and returns first row of response.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    /// 3) Query returns more than one row.
    pub fn fetch_row<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyList>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_fetch_row(querystring, params).await
        })
    }

    /// Execute querystring with parameters and return first value in the first row.
    ///
    /// It converts incoming parameters to rust readable,
    /// executes query with them and returns first row of response.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    /// 3) Query returns more than one row
    pub fn fetch_val<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyList>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let transaction_arc = self.transaction.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            let first_row = transaction_guard
                .inner_fetch_row(querystring, params)
                .await?
                .get_inner();
            Python::with_gil(|py| match first_row.columns().first() {
                Some(first_column) => postgres_to_py(py, &first_row, first_column, 0),
                None => Ok(py.None()),
            })
        })
    }

    /// Execute querystrings with parameters and return all results.
    ///
    /// Create pipeline of queries.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute any of querystring.
    pub fn pipeline<'a>(
        &'a self,
        py: Python<'a>,
        queries: Option<&'a PyList>,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let mut processed_queries: Vec<(String, Vec<PythonDTO>)> = vec![];
        if let Some(queries) = queries {
            for single_query in queries {
                let query_tuple = single_query.downcast::<PyTuple>().map_err(|err| {
                    RustPSQLDriverError::PyToRustValueConversionError(format!(
                        "Cannot cast to tuple: {err}",
                    ))
                })?;
                let querystring = query_tuple.get_item(0)?.extract::<String>()?;
                match query_tuple.get_item(1) {
                    Ok(params) => {
                        processed_queries.push((querystring, convert_parameters(params)?));
                    }
                    Err(_) => {
                        processed_queries.push((querystring, vec![]));
                    }
                }
            }
        }

        let transaction_arc = self.transaction.clone();

        rustengine_future(py, async move {
            let transaction_guard = transaction_arc.read().await;
            transaction_guard.inner_pipeline(processed_queries).await
        })
    }

    /// Start the transaction.
    ///
    /// # Errors
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
    /// # Errors
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
    /// # Errors
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
    /// # Errors
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
    /// # Errors
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
    /// # Errors
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

    /// Create new cursor.
    ///
    /// Call `inner_cursor` function.
    ///
    /// # Errors
    /// May return Err Result if can't convert incoming parameters
    /// or if `inner_cursor` returns error.
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
            params = convert_parameters(parameters)?;
        }

        rustengine_future(py, async move {
            let mut transaction_guard = transaction_arc.write().await;
            transaction_guard
                .inner_cursor(querystring, params, fetch_number.unwrap_or(10), scroll)
                .await
        })
    }
}
