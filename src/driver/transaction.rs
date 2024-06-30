use deadpool_postgres::Object;
use futures_util::future;
use pyo3::{
    prelude::*,
    pyclass,
    types::{PyList, PyTuple},
};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, postgres_to_py, PythonDTO, QueryParameter},
};

use super::{
    cursor::Cursor,
    transaction_options::{IsolationLevel, ReadVariant},
};
use crate::common::ObjectQueryTrait;
use std::{collections::HashSet, sync::Arc};

#[allow(clippy::module_name_repetitions)]
pub trait TransactionObjectTrait {
    fn start_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        defferable: Option<bool>,
    ) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
    fn commit(&self) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
    fn rollback(&self) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
}

impl TransactionObjectTrait for Object {
    async fn start_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let mut querystring = "START TRANSACTION".to_string();

        if let Some(level) = isolation_level {
            let level = &level.to_str_level();
            querystring.push_str(format!(" ISOLATION LEVEL {level}").as_str());
        };

        querystring.push_str(match read_variant {
            Some(ReadVariant::ReadOnly) => " READ ONLY",
            Some(ReadVariant::ReadWrite) => " READ WRITE",
            None => "",
        });

        querystring.push_str(match deferrable {
            Some(true) => " DEFERRABLE",
            Some(false) => " NOT DEFERRABLE",
            None => "",
        });
        self.batch_execute(&querystring).await.map_err(|err| {
            RustPSQLDriverError::TransactionBeginError(format!(
                "Cannot execute statement to start transaction, err - {err}"
            ))
        })?;

        Ok(())
    }
    async fn commit(&self) -> RustPSQLDriverPyResult<()> {
        self.batch_execute("COMMIT;").await.map_err(|err| {
            RustPSQLDriverError::TransactionCommitError(format!(
                "Cannot execute COMMIT statement, error - {err}"
            ))
        })?;
        Ok(())
    }
    async fn rollback(&self) -> RustPSQLDriverPyResult<()> {
        self.batch_execute("ROLLBACK;").await.map_err(|err| {
            RustPSQLDriverError::TransactionRollbackError(format!(
                "Cannot execute ROLLBACK statement, error - {err}"
            ))
        })?;
        Ok(())
    }
}

#[pyclass]
pub struct Transaction {
    pub db_client: Option<Arc<Object>>,
    is_started: bool,
    is_done: bool,

    isolation_level: Option<IsolationLevel>,
    read_variant: Option<ReadVariant>,
    deferrable: Option<bool>,

    savepoints_map: HashSet<String>,
}

impl Transaction {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        db_client: Arc<Object>,
        is_started: bool,
        is_done: bool,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
        savepoints_map: HashSet<String>,
    ) -> Self {
        Self {
            db_client: Some(db_client),
            is_started,
            is_done,
            isolation_level,
            read_variant,
            deferrable,
            savepoints_map,
        }
    }

    fn check_is_transaction_ready(&self) -> RustPSQLDriverPyResult<()> {
        if !self.is_started {
            return Err(RustPSQLDriverError::TransactionBeginError(
                "Transaction is not started, please call begin() on transaction".into(),
            ));
        }
        if self.is_done {
            return Err(RustPSQLDriverError::TransactionBeginError(
                "Transaction is already committed or rolled back".into(),
            ));
        }
        Ok(())
    }
}

#[pymethods]
impl Transaction {
    #[must_use]
    pub fn __aiter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __await__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    async fn __aenter__<'a>(self_: Py<Self>) -> RustPSQLDriverPyResult<Py<Self>> {
        let (is_started, is_done, isolation_level, read_variant, deferrable, db_client) =
            pyo3::Python::with_gil(|gil| {
                let self_ = self_.borrow(gil);
                (
                    self_.is_started,
                    self_.is_done,
                    self_.isolation_level,
                    self_.read_variant,
                    self_.deferrable,
                    self_.db_client.clone(),
                )
            });

        if is_started {
            return Err(RustPSQLDriverError::TransactionBeginError(
                "Transaction is already started".into(),
            ));
        }

        if is_done {
            return Err(RustPSQLDriverError::TransactionBeginError(
                "Transaction is already committed or rolled back".into(),
            ));
        }

        if let Some(db_client) = db_client {
            db_client
                .start_transaction(isolation_level, read_variant, deferrable)
                .await?;

            Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.is_started = true;
            });
            return Ok(self_);
        }

        Err(RustPSQLDriverError::TransactionClosedError)
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __aexit__<'a>(
        self_: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<()> {
        let (is_transaction_ready, is_exception_none, py_err, db_client) =
            pyo3::Python::with_gil(|gil| {
                let self_ = self_.borrow(gil);
                (
                    self_.check_is_transaction_ready(),
                    exception.is_none(gil),
                    PyErr::from_value_bound(exception.into_bound(gil)),
                    self_.db_client.clone(),
                )
            });
        is_transaction_ready?;

        if let Some(db_client) = db_client {
            let exit_result = if is_exception_none {
                db_client.commit().await?;
                Ok(())
            } else {
                db_client.rollback().await?;
                Err(RustPSQLDriverError::RustPyError(py_err))
            };

            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.is_done = true;
                std::mem::take(&mut self_.db_client);
            });
            return exit_result;
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn commit(&mut self) -> RustPSQLDriverPyResult<()> {
        self.check_is_transaction_ready()?;
        if let Some(db_client) = &self.db_client {
            db_client.commit().await?;
            self.is_done = true;
            std::mem::take(&mut self.db_client);
            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn rollback(&mut self) -> RustPSQLDriverPyResult<()> {
        self.check_is_transaction_ready()?;
        if let Some(db_client) = &self.db_client {
            db_client.rollback().await?;
            self.is_done = true;
            std::mem::take(&mut self.db_client);
            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn execute(
        self_: Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (is_transaction_ready, db_client) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (self_.check_is_transaction_ready(), self_.db_client.clone())
        });
        is_transaction_ready?;
        if let Some(db_client) = db_client {
            return db_client
                .psqlpy_query(querystring, parameters, prepared)
                .await;
        }

        Err(RustPSQLDriverError::TransactionClosedError)
    }

    /// Fetch result from the database.
    ///
    /// It converts incoming parameters to rust readable
    /// and then execute the query with them.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    pub async fn fetch(
        self_: Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (is_transaction_ready, db_client) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (self_.check_is_transaction_ready(), self_.db_client.clone())
        });
        is_transaction_ready?;
        if let Some(db_client) = db_client {
            return db_client
                .psqlpy_query(querystring, parameters, prepared)
                .await;
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn fetch_row(
        self_: Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
        let (is_transaction_ready, db_client) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (self_.check_is_transaction_ready(), self_.db_client.clone())
        });
        is_transaction_ready?;

        if let Some(db_client) = db_client {
            let mut params: Vec<PythonDTO> = vec![];
            if let Some(parameters) = parameters {
                params = convert_parameters(parameters)?;
            }

            let result = if prepared.unwrap_or(true) {
                db_client
                    .query_one(
                        &db_client.prepare_cached(&querystring).await?,
                        &params
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await?
            } else {
                db_client
                    .query_one(
                        &querystring,
                        &params
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await?
            };

            return Ok(PSQLDriverSinglePyQueryResult::new(result));
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn fetch_val(
        self_: Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let (is_transaction_ready, db_client) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (self_.check_is_transaction_ready(), self_.db_client.clone())
        });
        if let Some(db_client) = db_client {
            is_transaction_ready?;
            let mut params: Vec<PythonDTO> = vec![];
            if let Some(parameters) = parameters {
                params = convert_parameters(parameters)?;
            }

            let result = if prepared.unwrap_or(true) {
                db_client
                    .query_one(
                        &db_client.prepare_cached(&querystring).await?,
                        &params
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await?
            } else {
                db_client
                    .query_one(
                        &querystring,
                        &params
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await?
            };

            return Python::with_gil(|gil| match result.columns().first() {
                Some(first_column) => postgres_to_py(gil, &result, first_column, 0, &None),
                None => Ok(gil.None()),
            });
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn execute_many(
        self_: Py<Self>,
        querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let (is_transaction_ready, db_client) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (self_.check_is_transaction_ready(), self_.db_client.clone())
        });

        if let Some(db_client) = db_client {
            is_transaction_ready?;

            let mut params: Vec<Vec<PythonDTO>> = vec![];
            if let Some(parameters) = parameters {
                for vec_of_py_any in parameters {
                    params.push(convert_parameters(vec_of_py_any)?);
                }
            }
            let prepared = prepared.unwrap_or(true);

            for param in params {
                let is_query_result_ok = if prepared {
                    let prepared_stmt = &db_client.prepare_cached(&querystring).await;
                    if let Err(error) = prepared_stmt {
                        return Err(RustPSQLDriverError::TransactionExecuteError(format!(
                            "Cannot prepare statement in execute_many, operation rolled back {error}",
                        )));
                    }
                    db_client
                        .query(
                            &db_client.prepare_cached(&querystring).await?,
                            &param
                                .iter()
                                .map(|param| param as &QueryParameter)
                                .collect::<Vec<&QueryParameter>>()
                                .into_boxed_slice(),
                        )
                        .await
                } else {
                    db_client
                        .query(
                            &querystring,
                            &param
                                .iter()
                                .map(|param| param as &QueryParameter)
                                .collect::<Vec<&QueryParameter>>()
                                .into_boxed_slice(),
                        )
                        .await
                };
                is_query_result_ok?;
            }

            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn begin(self_: Py<Self>) -> RustPSQLDriverPyResult<()> {
        let (is_started, is_done, isolation_level, read_variant, deferrable, db_client) =
            pyo3::Python::with_gil(|gil| {
                let self_ = self_.borrow(gil);
                (
                    self_.is_started,
                    self_.is_done,
                    self_.isolation_level,
                    self_.read_variant,
                    self_.deferrable,
                    self_.db_client.clone(),
                )
            });

        if let Some(db_client) = db_client {
            if is_started {
                return Err(RustPSQLDriverError::TransactionBeginError(
                    "Transaction is already started".into(),
                ));
            }

            if is_done {
                return Err(RustPSQLDriverError::TransactionBeginError(
                    "Transaction is already committed or rolled back".into(),
                ));
            }
            db_client
                .start_transaction(isolation_level, read_variant, deferrable)
                .await?;

            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.is_started = true;
            });

            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn create_savepoint(
        self_: Py<Self>,
        savepoint_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let (is_transaction_ready, is_savepoint_name_exists, db_client) =
            pyo3::Python::with_gil(|gil| {
                let self_ = self_.borrow(gil);
                (
                    self_.check_is_transaction_ready(),
                    self_.savepoints_map.contains(&savepoint_name),
                    self_.db_client.clone(),
                )
            });

        if let Some(db_client) = db_client {
            is_transaction_ready?;

            if is_savepoint_name_exists {
                return Err(RustPSQLDriverError::TransactionSavepointError(format!(
                    "SAVEPOINT name {savepoint_name} is already taken by this transaction",
                )));
            }
            db_client
                .batch_execute(format!("SAVEPOINT {savepoint_name}").as_str())
                .await?;

            pyo3::Python::with_gil(|gil| {
                self_.borrow_mut(gil).savepoints_map.insert(savepoint_name);
            });
            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn release_savepoint(
        self_: Py<Self>,
        savepoint_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let (is_transaction_ready, is_savepoint_name_exists, db_client) =
            pyo3::Python::with_gil(|gil| {
                let self_ = self_.borrow(gil);
                (
                    self_.check_is_transaction_ready(),
                    self_.savepoints_map.contains(&savepoint_name),
                    self_.db_client.clone(),
                )
            });

        if let Some(db_client) = db_client {
            is_transaction_ready?;

            if !is_savepoint_name_exists {
                return Err(RustPSQLDriverError::TransactionSavepointError(
                    "Don't have rollback with this name".into(),
                ));
            }
            db_client
                .batch_execute(format!("RELEASE SAVEPOINT {savepoint_name}").as_str())
                .await?;

            pyo3::Python::with_gil(|gil| {
                self_.borrow_mut(gil).savepoints_map.remove(&savepoint_name);
            });
            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn rollback_savepoint(
        self_: Py<Self>,
        savepoint_name: String,
    ) -> RustPSQLDriverPyResult<()> {
        let (is_transaction_ready, is_savepoint_name_exists, db_client) =
            pyo3::Python::with_gil(|gil| {
                let self_ = self_.borrow(gil);
                (
                    self_.check_is_transaction_ready(),
                    self_.savepoints_map.contains(&savepoint_name),
                    self_.db_client.clone(),
                )
            });

        if let Some(db_client) = db_client {
            is_transaction_ready?;

            if !is_savepoint_name_exists {
                return Err(RustPSQLDriverError::TransactionSavepointError(
                    "Don't have rollback with this name".into(),
                ));
            }
            db_client
                .batch_execute(format!("ROLLBACK TO SAVEPOINT {savepoint_name}").as_str())
                .await?;

            pyo3::Python::with_gil(|gil| {
                self_.borrow_mut(gil).savepoints_map.remove(&savepoint_name);
            });
            return Ok(());
        }

        Err(RustPSQLDriverError::TransactionClosedError)
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
    pub async fn pipeline<'py>(
        self_: Py<Self>,
        queries: Option<Py<PyList>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<Vec<PSQLDriverPyQueryResult>> {
        let (is_transaction_ready, db_client) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);

            (self_.check_is_transaction_ready(), self_.db_client.clone())
        });

        if let Some(db_client) = db_client {
            is_transaction_ready?;

            let mut futures = vec![];
            if let Some(queries) = queries {
                let gil_result = pyo3::Python::with_gil(|gil| -> PyResult<()> {
                    for single_query in queries.into_bound(gil).iter() {
                        let query_tuple = single_query.downcast::<PyTuple>().map_err(|err| {
                            RustPSQLDriverError::PyToRustValueConversionError(format!(
                                "Cannot cast to tuple: {err}",
                            ))
                        })?;

                        let querystring = query_tuple.get_item(0)?.extract::<String>()?;
                        let params = match query_tuple.get_item(1) {
                            Ok(param) => Some(param.into()),
                            Err(_) => None,
                        };
                        futures.push(db_client.psqlpy_query(querystring, params, prepared));
                    }
                    Ok(())
                });

                match gil_result {
                    Ok(()) => {}
                    Err(e) => {
                        // Handle PyO3 error, convert to your error type as needed
                        return Err(RustPSQLDriverError::from(e)); // Adjust according to your error types
                    }
                }
            }
            return future::try_join_all(futures).await;
        }

        Err(RustPSQLDriverError::TransactionClosedError)
    }

    /// Create new cursor object.
    ///
    /// # Errors
    /// May return Err Result if db_client is None
    pub fn cursor(
        &self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        fetch_number: Option<usize>,
        scroll: Option<bool>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<Cursor> {
        if let Some(db_client) = &self.db_client {
            return Ok(Cursor::new(
                db_client.clone(),
                querystring,
                parameters,
                "cur_name".into(),
                fetch_number.unwrap_or(10),
                scroll,
                prepared,
            ));
        }

        Err(RustPSQLDriverError::TransactionClosedError)
    }
}
