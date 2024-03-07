use deadpool_postgres::Object;
use pyo3::{exceptions::PyStopAsyncIteration, pyclass, pymethods, PyAny, PyObject, PyRef, Python};
use std::sync::Arc;

use crate::{
    common::rustengine_future, exceptions::rust_errors::RustPSQLDriverPyResult,
    query_result::PSQLDriverPyQueryResult,
};

#[pyclass]
pub struct Cursor {
    db_client: Arc<tokio::sync::RwLock<Object>>,
    cursor_name: String,
    fetch_number: usize,
    closed: Arc<tokio::sync::RwLock<bool>>,
}

impl Cursor {
    pub fn new(
        db_client: Arc<tokio::sync::RwLock<Object>>,
        cursor_name: String,
        fetch_number: usize,
    ) -> Self {
        Cursor {
            db_client,
            cursor_name,
            fetch_number,
            closed: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }
}

#[pymethods]
impl Cursor {
    #[must_use]
    pub fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Return next result from the SQL statement.
    ///
    /// Execute FETCH <number> FROM <cursor name>
    ///
    /// # Errors
    /// May return Err Result if can't execute querystring.
    pub fn __anext__(&self, py: Python<'_>) -> RustPSQLDriverPyResult<Option<PyObject>> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();
        let fetch_number = self.fetch_number;

        let future = rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH {fetch_number} FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;

            if result.is_empty() {
                return Err(PyStopAsyncIteration::new_err(
                    "Iteration is over, no more results in cursor",
                )
                .into());
            };

            Ok(PSQLDriverPyQueryResult::new(result))
        });

        Ok(Some(future?.into()))
    }

    /// Fetch data from cursor.
    ///
    /// It's possible to specify fetch number.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch<'a>(
        &'a self,
        py: Python<'a>,
        fetch_number: Option<usize>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();
        let fetch_number = match fetch_number {
            Some(usize) => usize,
            None => self.fetch_number,
        };

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH {fetch_number} FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch row from cursor.
    ///
    /// Execute FETCH NEXT.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_next<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(format!("FETCH NEXT FROM {cursor_name}").as_str(), &[])
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch previous from cursor.
    ///
    /// Execute FETCH PRIOR.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_prior<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(format!("FETCH PRIOR FROM {cursor_name}").as_str(), &[])
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch first row from cursor.
    ///
    /// Execute FETCH FIRST (same as ABSOLUTE 1)
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_first<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(format!("FETCH FIRST FROM {cursor_name}").as_str(), &[])
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch last row from cursor.
    ///
    /// Execute FETCH LAST (same as ABSOLUTE -1)
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_last<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(format!("FETCH LAST FROM {cursor_name}").as_str(), &[])
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch absolute row from cursor.
    ///
    /// Execute FETCH ABSOLUTE<absolute_number>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_absolute<'a>(
        &'a self,
        py: Python<'a>,
        absolute_number: i64,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH ABSOLUTE {absolute_number} FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch relative row from cursor.
    ///
    /// Execute FETCH RELATIVE<absolute_number>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_relative<'a>(
        &'a self,
        py: Python<'a>,
        relative_number: i64,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH RELATIVE {relative_number} FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch forward all from cursor.
    ///
    /// Execute FORWARD ALL.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_forward_all<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH FORWARD ALL FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch backward from cursor.
    ///
    /// Execute BACKWARD <backward_count>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_backward<'a>(
        &'a self,
        py: Python<'a>,
        backward_count: i64,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH BACKWARD {backward_count} FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Fetch backward from cursor.
    ///
    /// Execute BACKWARD <backward_count>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub fn fetch_backward_all<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();

        rustengine_future(py, async move {
            let db_client_guard = db_client_arc.read().await;
            let result = db_client_guard
                .query(
                    format!("FETCH BACKWARD ALL FROM {cursor_name}").as_str(),
                    &[],
                )
                .await?;
            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    /// Close cursor.
    ///
    /// # Errors
    /// May return Err Result if cannot execute CLOSE command
    pub fn close<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();
        let cursor_name = self.cursor_name.clone();
        let closed = self.closed.clone();

        rustengine_future(py, async move {
            let is_closed = {
                let closed_read = closed.write().await;
                *closed_read
            };
            if is_closed {
                return Err(
                    crate::exceptions::rust_errors::RustPSQLDriverError::DBCursorError(
                        "Cursor is already closed".into(),
                    ),
                );
            }

            let db_client_guard = db_client_arc.read().await;

            db_client_guard
                .batch_execute(format!("CLOSE {cursor_name}").as_str())
                .await?;

            let mut closed_write = closed.write().await;
            *closed_write = true;

            Ok(())
        })
    }
}
