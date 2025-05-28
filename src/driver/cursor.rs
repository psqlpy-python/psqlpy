use std::sync::Arc;

use pyo3::{
    exceptions::PyStopAsyncIteration, pyclass, pymethods, Py, PyAny, PyErr, PyObject, Python,
};
use tokio::sync::RwLock;
use tokio_postgres::{Config, Portal as tp_Portal};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    query_result::PSQLDriverPyQueryResult,
    runtime::rustdriver_future,
    statement::statement::PsqlpyStatement,
    transaction::structs::PSQLPyTransaction,
};

use crate::connection::structs::PSQLPyConnection;

#[pyclass]
pub struct Cursor {
    pub conn: Option<Arc<RwLock<PSQLPyConnection>>>,
    querystring: Option<String>,
    parameters: Option<Py<PyAny>>,
    array_size: i32,

    statement: Option<PsqlpyStatement>,

    transaction: Option<Arc<PSQLPyTransaction>>,
    inner: Option<tp_Portal>,

    pub pg_config: Arc<Config>,
}

impl Cursor {
    pub fn new(
        conn: Option<Arc<RwLock<PSQLPyConnection>>>,
        querystring: Option<String>,
        parameters: Option<Py<PyAny>>,
        array_size: Option<i32>,
        pg_config: Arc<Config>,
        statement: Option<PsqlpyStatement>,
    ) -> Self {
        Self {
            conn,
            transaction: None,
            inner: None,
            querystring,
            parameters,
            array_size: array_size.unwrap_or(1),
            pg_config,
            statement,
        }
    }

    async fn query_portal(&self, size: i32) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(transaction) = &self.transaction else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let Some(portal) = &self.inner else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        transaction.query_portal(&portal, size).await
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        self.transaction = None;
        self.conn = None;
    }
}

#[pymethods]
impl Cursor {
    #[getter]
    fn get_array_size(&self) -> i32 {
        self.array_size
    }

    #[setter]
    fn set_array_size(&mut self, value: i32) {
        self.array_size = value;
    }

    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __await__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    async fn __aenter__<'a>(slf: Py<Self>) -> PSQLPyResult<Py<Self>> {
        let (conn, querystring, parameters, statement) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (
                self_.conn.clone(),
                self_.querystring.clone(),
                self_.parameters.clone(),
                self_.statement.clone(),
            )
        });

        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        let mut write_conn_g = conn.write().await;

        let (txid, inner_portal) = match querystring {
            Some(querystring) => {
                write_conn_g
                    .portal(Some(&querystring), &parameters, None)
                    .await?
            }
            None => {
                let Some(statement) = statement else {
                    return Err(RustPSQLDriverError::CursorStartError(
                        "Cannot start cursor".into(),
                    ));
                };
                write_conn_g.portal(None, &None, Some(&statement)).await?
            }
        };

        Python::with_gil(|gil| {
            let mut self_ = slf.borrow_mut(gil);

            self_.transaction = Some(Arc::new(txid));
            self_.inner = Some(inner_portal);
        });

        Ok(slf)
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __aexit__<'a>(
        &mut self,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> PSQLPyResult<()> {
        self.close();

        let (is_exc_none, py_err) = pyo3::Python::with_gil(|gil| {
            (
                exception.is_none(gil),
                PyErr::from_value(exception.into_bound(gil)),
            )
        });

        if !is_exc_none {
            return Err(RustPSQLDriverError::RustPyError(py_err));
        }
        Ok(())
    }

    fn __anext__(&self) -> PSQLPyResult<Option<PyObject>> {
        let txid = self.transaction.clone();
        let portal = self.inner.clone();
        let size = self.array_size.clone();

        let py_future = Python::with_gil(move |gil| {
            rustdriver_future(gil, async move {
                let Some(txid) = &txid else {
                    return Err(RustPSQLDriverError::TransactionClosedError);
                };
                let Some(portal) = &portal else {
                    return Err(RustPSQLDriverError::TransactionClosedError);
                };
                let result = txid.query_portal(&portal, size).await?;

                if result.is_empty() {
                    return Err(PyStopAsyncIteration::new_err(
                        "Iteration is over, no more results in portal",
                    )
                    .into());
                };

                Ok(result)
            })
        });

        Ok(Some(py_future?))
    }

    async fn start(&mut self) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::ConnectionClosedError);
        };
        let mut write_conn_g = conn.write().await;

        let (txid, inner_portal) = match &self.querystring {
            Some(querystring) => {
                write_conn_g
                    .portal(Some(&querystring), &self.parameters, None)
                    .await?
            }
            None => {
                let Some(statement) = &self.statement else {
                    return Err(RustPSQLDriverError::CursorStartError(
                        "Cannot start cursor".into(),
                    ));
                };
                write_conn_g.portal(None, &None, Some(&statement)).await?
            }
        };

        self.transaction = Some(Arc::new(txid));
        self.inner = Some(inner_portal);

        Ok(())
    }

    fn close(&mut self) {
        self.transaction = None;
        self.conn = None;
    }

    #[pyo3(signature = (
        querystring,
        parameters=None,
    ))]
    async fn execute(
        &mut self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
    ) -> PSQLPyResult<()> {
        self.querystring = Some(querystring);
        self.parameters = parameters;

        self.start().await?;

        Ok(())
    }

    async fn fetchone(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        self.query_portal(1).await
    }

    #[pyo3(signature = (size=None))]
    async fn fetchmany(&self, size: Option<i32>) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        self.query_portal(size.unwrap_or(self.array_size)).await
    }

    async fn fetchall(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        self.query_portal(-1).await
    }
}
