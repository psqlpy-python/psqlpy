use std::sync::Arc;

use futures::future;
use pyo3::{
    pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyTuple},
    Py, PyAny, PyErr, PyResult,
};
use tokio::sync::RwLock;
use tokio_postgres::Config;

use crate::{
    connection::{
        structs::PSQLPyConnection,
        traits::{CloseTransaction, Connection, StartTransaction as _},
    },
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    options::{IsolationLevel, ReadVariant},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
};

#[pyclass(subclass)]
#[derive(Debug)]
pub struct Transaction {
    pub conn: Option<Arc<RwLock<PSQLPyConnection>>>,
    pub pg_config: Arc<Config>,

    isolation_level: Option<IsolationLevel>,
    read_variant: Option<ReadVariant>,
    deferrable: Option<bool>,
}

impl Transaction {
    #[must_use]
    pub fn new(
        conn: Option<Arc<RwLock<PSQLPyConnection>>>,
        pg_config: Arc<Config>,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> Self {
        Self {
            conn,
            pg_config,
            isolation_level,
            read_variant,
            deferrable,
        }
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

    async fn __aenter__(self_: Py<Self>) -> PSQLPyResult<Py<Self>> {
        let (isolation_level, read_variant, deferrable, conn) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (
                self_.isolation_level,
                self_.read_variant,
                self_.deferrable,
                self_.conn.clone(),
            )
        });

        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        write_conn_g
            .start_transaction(isolation_level, read_variant, deferrable)
            .await?;

        Ok(self_)
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __aexit__(
        self_: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> PSQLPyResult<()> {
        let (conn, is_exception_none, py_err) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (
                self_.conn.clone(),
                exception.is_none(gil),
                PyErr::from_value(exception.into_bound(gil)),
            )
        });

        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        if is_exception_none {
            write_conn_g.commit().await?;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.conn = None;
            });
            Ok(())
        } else {
            write_conn_g.rollback().await?;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.conn = None;
            });
            Err(RustPSQLDriverError::RustPyError(py_err))
        }
    }

    /// Begin the transaction.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    pub async fn begin(&mut self) -> PSQLPyResult<()> {
        let conn = &self.conn;
        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        write_conn_g
            .start_transaction(self.isolation_level, self.read_variant, self.deferrable)
            .await?;

        Ok(())
    }

    /// Execute querystring with parameters.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g.execute(querystring, parameters, prepared).await
    }

    /// Execute querystring with parameters.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g.execute(querystring, parameters, prepared).await
    }

    /// Execute querystring with parameters and return single value.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    /// Or if query returns more than one value.
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch_val(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .fetch_val(querystring, parameters, prepared)
            .await
    }

    /// Executes a sequence of SQL statements using the simple query protocol.
    ///
    /// Statements should be separated by semicolons.
    /// If an error occurs, execution of the sequence will stop at that point.
    /// This is intended for use when, for example, initializing a database schema.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    pub async fn execute_batch(&self, querystring: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g.batch_execute(&querystring).await
    }

    /// Executes one query with different parameters.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute_many(
        &self,
        querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .execute_many(querystring, parameters, prepared)
            .await
    }

    /// Executes query and return one row.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch_row(
        &self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverSinglePyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .fetch_row(querystring, parameters, prepared)
            .await
    }

    /// Create new savepoint in a transaction.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    pub async fn create_savepoint(&mut self, savepoint_name: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .batch_execute(format!("SAVEPOINT {savepoint_name}").as_str())
            .await?;

        Ok(())
    }

    /// Release a savepoint in a transaction.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    pub async fn release_savepoint(&mut self, savepoint_name: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .batch_execute(format!("RELEASE SAVEPOINT {savepoint_name}").as_str())
            .await?;

        Ok(())
    }

    /// Rollback to a savepoint in a transaction.
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    pub async fn rollback_savepoint(&mut self, savepoint_name: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .batch_execute(format!("ROLLBACK TO SAVEPOINT {savepoint_name}").as_str())
            .await?;

        Ok(())
    }

    /// Execute many queries in a transaction.
    ///
    /// More information in a documentation:
    /// https://psqlpy-python.github.io/components/transaction.html#pipeline
    ///
    /// # Errors
    /// Can return error if there is a problem with DB communication.
    #[allow(for_loops_over_fallibles)]
    #[pyo3(signature = (queries=None, prepared=None))]
    pub async fn pipeline(
        self_: Py<Self>,
        queries: Option<Py<PyList>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Vec<PSQLDriverPyQueryResult>> {
        let db_client = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);

            self_.conn.clone()
        });

        if let Some(db_client) = db_client {
            let conn_read_g = db_client.read().await;
            let mut futures = vec![];
            if let Some(queries) = queries {
                let gil_result = pyo3::Python::with_gil(|gil| -> PyResult<()> {
                    for single_query in queries.into_bound(gil).try_iter() {
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
                        futures.push(conn_read_g.execute(querystring, params, prepared));
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
}
