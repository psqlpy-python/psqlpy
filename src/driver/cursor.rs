use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use pyo3::{
    exceptions::PyStopAsyncIteration, pyclass, pymethods, Py, PyAny, PyErr, PyObject, Python,
};
use tokio::sync::RwLock;
use tokio_postgres::Config;

use crate::{
    connection::{structs::PSQLPyConnection, traits::Cursor as _},
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    query_result::PSQLDriverPyQueryResult,
    runtime::rustdriver_future,
};

static NEXT_CUR_ID: AtomicUsize = AtomicUsize::new(0);

fn next_cursor_name() -> String {
    format!("cur{}", NEXT_CUR_ID.fetch_add(1, Ordering::SeqCst),)
}

#[pyclass(subclass)]
pub struct Cursor {
    conn: Option<Arc<RwLock<PSQLPyConnection>>>,
    pub pg_config: Arc<Config>,
    querystring: String,
    parameters: Option<Py<PyAny>>,
    cursor_name: Option<String>,
    fetch_number: usize,
    scroll: Option<bool>,
    prepared: Option<bool>,
}

impl Cursor {
    pub fn new(
        conn: Arc<RwLock<PSQLPyConnection>>,
        pg_config: Arc<Config>,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        fetch_number: usize,
        scroll: Option<bool>,
        prepared: Option<bool>,
    ) -> Self {
        Cursor {
            conn: Some(conn),
            pg_config,
            querystring,
            parameters,
            cursor_name: None,
            fetch_number,
            scroll,
            prepared,
        }
    }

    async fn execute(&self, querystring: &str) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        let read_conn_g = conn.read().await;

        let result = read_conn_g
            .execute(querystring.to_string(), None, Some(false))
            .await
            .map_err(|err| {
                RustPSQLDriverError::CursorFetchError(format!(
                    "Cannot fetch data from cursor, error - {err}"
                ))
            })?;

        Ok(result)
    }
}

#[pymethods]
impl Cursor {
    #[getter]
    fn cursor_name(&self) -> Option<String> {
        return self.cursor_name.clone();
    }

    #[getter]
    fn querystring(&self) -> String {
        return self.querystring.clone();
    }

    #[getter]
    fn parameters(&self) -> Option<Py<PyAny>> {
        return self.parameters.clone();
    }

    #[getter]
    fn prepared(&self) -> Option<bool> {
        return self.prepared.clone();
    }

    #[must_use]
    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __await__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    async fn __aenter__<'a>(slf: Py<Self>) -> PSQLPyResult<Py<Self>> {
        let cursor_name = next_cursor_name();

        let (conn, scroll, querystring, prepared, parameters) = Python::with_gil(|gil| {
            let mut self_ = slf.borrow_mut(gil);
            self_.cursor_name = Some(cursor_name.clone());
            (
                self_.conn.clone(),
                self_.scroll,
                self_.querystring.clone(),
                self_.prepared,
                self_.parameters.clone(),
            )
        });

        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        let mut write_conn_g = conn.write().await;

        write_conn_g
            .start_cursor(
                &cursor_name,
                &scroll,
                querystring.clone(),
                &prepared,
                parameters.clone(),
            )
            .await?;

        Ok(slf)
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __aexit__<'a>(
        &mut self,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> PSQLPyResult<()> {
        self.close().await?;

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
        let conn = self.conn.clone();
        let fetch_number = self.fetch_number;
        let Some(cursor_name) = self.cursor_name.clone() else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };

        let py_future = Python::with_gil(move |gil| {
            rustdriver_future(gil, async move {
                let Some(conn) = conn else {
                    return Err(RustPSQLDriverError::CursorClosedError);
                };

                let read_conn_g = conn.read().await;
                let result = read_conn_g
                    .execute(
                        format!("FETCH {fetch_number} FROM {cursor_name}"),
                        None,
                        Some(false),
                    )
                    .await?;

                if result.is_empty() {
                    return Err(PyStopAsyncIteration::new_err(
                        "Iteration is over, no more results in cursor",
                    )
                    .into());
                };
                Ok(result)
            })
        });

        Ok(Some(py_future?))
    }

    pub async fn start(&mut self) -> PSQLPyResult<()> {
        if self.cursor_name.is_some() {
            return Ok(());
        }

        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        let mut write_conn_g = conn.write().await;

        let cursor_name = next_cursor_name();

        write_conn_g
            .start_cursor(
                &cursor_name,
                &self.scroll,
                self.querystring.clone(),
                &self.prepared,
                self.parameters.clone(),
            )
            .await?;

        self.cursor_name = Some(cursor_name);

        Ok(())
    }

    pub async fn close(&mut self) -> PSQLPyResult<()> {
        if let Some(cursor_name) = &self.cursor_name {
            let Some(conn) = &self.conn else {
                return Err(RustPSQLDriverError::CursorClosedError);
            };
            let mut write_conn_g = conn.write().await;
            write_conn_g.close_cursor(&cursor_name).await?;
            self.cursor_name = None;
        };

        self.conn = None;

        Ok(())
    }

    #[pyo3(signature = (fetch_number=None))]
    pub async fn fetch(
        &self,
        fetch_number: Option<usize>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!(
            "FETCH {} FROM {}",
            fetch_number.unwrap_or(self.fetch_number),
            cursor_name,
        ))
        .await
    }

    pub async fn fetch_next(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!("FETCH NEXT FROM {cursor_name}"))
            .await
    }

    pub async fn fetch_prior(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!("FETCH PRIOR FROM {cursor_name}"))
            .await
    }

    pub async fn fetch_first(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!("FETCH FIRST FROM {cursor_name}"))
            .await
    }

    pub async fn fetch_last(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!("FETCH LAST FROM {cursor_name}"))
            .await
    }

    pub async fn fetch_absolute(
        &self,
        absolute_number: i64,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!(
            "FETCH ABSOLUTE {absolute_number} FROM {cursor_name}"
        ))
        .await
    }

    pub async fn fetch_relative(
        &self,
        relative_number: i64,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!(
            "FETCH RELATIVE {relative_number} FROM {cursor_name}"
        ))
        .await
    }

    pub async fn fetch_forward_all(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!("FETCH FORWARD ALL FROM {cursor_name}"))
            .await
    }

    pub async fn fetch_backward(
        &self,
        backward_count: i64,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!(
            "FETCH BACKWARD {backward_count} FROM {cursor_name}"
        ))
        .await
    }

    pub async fn fetch_backward_all(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(cursor_name) = &self.cursor_name else {
            return Err(RustPSQLDriverError::CursorClosedError);
        };
        self.execute(&format!("FETCH BACKWARD ALL FROM {cursor_name}"))
            .await
    }
}
