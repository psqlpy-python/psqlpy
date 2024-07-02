use std::sync::Arc;

use deadpool_postgres::Object;
use pyo3::{
    exceptions::PyStopAsyncIteration, pyclass, pymethods, Py, PyAny, PyErr, PyObject, Python,
};

use crate::{
    common::ObjectQueryTrait,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    runtime::rustdriver_future,
};

/// Additional implementation for the `Object` type.
trait CursorObjectTrait {
    async fn cursor_start(
        &self,
        cursor_name: &str,
        scroll: &Option<bool>,
        querystring: &str,
        prepared: &Option<bool>,
        parameters: &Option<Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<()>;

    async fn cursor_close(&self, closed: &bool, cursor_name: &str) -> RustPSQLDriverPyResult<()>;
}

impl CursorObjectTrait for Object {
    /// Start the cursor.
    ///
    /// Execute `DECLARE` command with parameters.
    ///
    /// # Errors
    /// May return Err Result if cannot execute querystring.
    async fn cursor_start(
        &self,
        cursor_name: &str,
        scroll: &Option<bool>,
        querystring: &str,
        prepared: &Option<bool>,
        parameters: &Option<Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<()> {
        let mut cursor_init_query = format!("DECLARE {cursor_name}");
        if let Some(scroll) = scroll {
            if *scroll {
                cursor_init_query.push_str(" SCROLL");
            } else {
                cursor_init_query.push_str(" NO SCROLL");
            }
        }

        cursor_init_query.push_str(format!(" CURSOR FOR {querystring}").as_str());

        self.psqlpy_query(cursor_init_query, parameters.clone(), *prepared)
            .await
            .map_err(|err| {
                RustPSQLDriverError::CursorStartError(format!("Cannot start cursor, error - {err}"))
            })?;

        Ok(())
    }

    /// Close the cursor.
    ///
    /// Execute `CLOSE` command.
    ///
    /// # Errors
    /// May return Err Result if cannot execute querystring.
    async fn cursor_close(&self, closed: &bool, cursor_name: &str) -> RustPSQLDriverPyResult<()> {
        if *closed {
            return Err(RustPSQLDriverError::CursorCloseError(
                "Cursor is already closed".into(),
            ));
        }

        self.psqlpy_query(
            format!("CLOSE {cursor_name}"),
            Option::default(),
            Some(false),
        )
        .await?;

        Ok(())
    }
}

#[pyclass]
pub struct Cursor {
    db_transaction: Option<Arc<Object>>,
    querystring: String,
    parameters: Option<Py<PyAny>>,
    cursor_name: String,
    fetch_number: usize,
    scroll: Option<bool>,
    prepared: Option<bool>,
    is_started: bool,
    closed: bool,
}

impl Cursor {
    #[must_use]
    pub fn new(
        db_transaction: Arc<Object>,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        cursor_name: String,
        fetch_number: usize,
        scroll: Option<bool>,
        prepared: Option<bool>,
    ) -> Self {
        Cursor {
            db_transaction: Some(db_transaction),
            querystring,
            parameters,
            cursor_name,
            fetch_number,
            scroll,
            prepared,
            is_started: false,
            closed: false,
        }
    }
}

#[pymethods]
impl Cursor {
    #[must_use]
    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __await__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    async fn __aenter__<'a>(slf: Py<Self>) -> RustPSQLDriverPyResult<Py<Self>> {
        let (db_transaction, cursor_name, scroll, querystring, prepared, parameters) =
            Python::with_gil(|gil| {
                let self_ = slf.borrow(gil);
                (
                    self_.db_transaction.clone(),
                    self_.cursor_name.clone(),
                    self_.scroll,
                    self_.querystring.clone(),
                    self_.prepared,
                    self_.parameters.clone(),
                )
            });

        if let Some(db_transaction) = db_transaction {
            db_transaction
                .cursor_start(&cursor_name, &scroll, &querystring, &prepared, &parameters)
                .await?;
            Python::with_gil(|gil| {
                let mut self_ = slf.borrow_mut(gil);
                self_.is_started = true;
            });
            return Ok(slf);
        }
        Err(RustPSQLDriverError::CursorClosedError)
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __aexit__<'a>(
        slf: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<()> {
        let (db_transaction, closed, cursor_name, is_exception_none, py_err) =
            pyo3::Python::with_gil(|gil| {
                let self_ = slf.borrow(gil);
                (
                    self_.db_transaction.clone(),
                    self_.closed,
                    self_.cursor_name.clone(),
                    exception.is_none(gil),
                    PyErr::from_value_bound(exception.into_bound(gil)),
                )
            });

        if let Some(db_transaction) = db_transaction {
            db_transaction
                .cursor_close(&closed, &cursor_name)
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorCloseError(format!(
                        "Cannot close the cursor, error - {err}"
                    ))
                })?;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = slf.borrow_mut(gil);
                std::mem::take(&mut self_.db_transaction);
            });
            if !is_exception_none {
                return Err(RustPSQLDriverError::RustPyError(py_err));
            }
            return Ok(());
        }
        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Return next result from the SQL statement.
    ///
    /// Execute FETCH <number> FROM <cursor name>
    ///
    /// This is the only place where we use `rustdriver_future` cuz
    /// we didn't find any solution how to implement it without
    /// # Errors
    /// May return Err Result if can't execute querystring.
    fn __anext__(&self) -> RustPSQLDriverPyResult<Option<PyObject>> {
        let db_transaction = self.db_transaction.clone();
        let fetch_number = self.fetch_number;
        let cursor_name = self.cursor_name.clone();

        let py_future = Python::with_gil(move |gil| {
            rustdriver_future(gil, async move {
                if let Some(db_transaction) = db_transaction {
                    let result = db_transaction
                        .psqlpy_query(
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

                    return Ok(result);
                }
                Err(RustPSQLDriverError::CursorClosedError)
            })
        });

        Ok(Some(py_future?))
    }

    /// Start the cursor
    ///
    /// # Errors
    /// May return Err Result
    /// if cannot execute querystring for cursor declaration.
    pub async fn start(&mut self) -> RustPSQLDriverPyResult<()> {
        let db_transaction_arc = self.db_transaction.clone();

        if let Some(db_transaction) = db_transaction_arc {
            db_transaction
                .cursor_start(
                    &self.cursor_name,
                    &self.scroll,
                    &self.querystring,
                    &self.prepared,
                    &self.parameters,
                )
                .await?;

            self.is_started = true;
            return Ok(());
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Close the cursor.
    ///
    /// It executes CLOSE command to close cursor in the transaction.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn close(&mut self) -> RustPSQLDriverPyResult<()> {
        let db_transaction_arc = self.db_transaction.clone();

        if let Some(db_transaction) = db_transaction_arc {
            db_transaction
                .cursor_close(&self.closed, &self.cursor_name)
                .await?;

            self.closed = true;
            std::mem::take(&mut self.db_transaction);
            return Ok(());
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch data from cursor.
    ///
    /// It's possible to specify fetch number.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch<'a>(
        slf: Py<Self>,
        fetch_number: Option<usize>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, inner_fetch_number, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (
                self_.db_transaction.clone(),
                self_.fetch_number,
                self_.cursor_name.clone(),
            )
        });

        if let Some(db_transaction) = db_transaction {
            let fetch_number = match fetch_number {
                Some(usize) => usize,
                None => inner_fetch_number,
            };

            let result = db_transaction
                .psqlpy_query(
                    format!("FETCH {fetch_number} FROM {cursor_name}"),
                    None,
                    Some(false),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;

            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch row from cursor.
    ///
    /// Execute FETCH NEXT.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_next<'a>(slf: Py<Self>) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(format!("FETCH NEXT FROM {cursor_name}"), None, Some(false))
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch previous from cursor.
    ///
    /// Execute FETCH PRIOR.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_prior<'a>(slf: Py<Self>) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(format!("FETCH PRIOR FROM {cursor_name}"), None, Some(false))
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch first row from cursor.
    ///
    /// Execute FETCH FIRST (same as ABSOLUTE 1)
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_first<'a>(slf: Py<Self>) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(format!("FETCH FIRST FROM {cursor_name}"), None, Some(false))
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch last row from cursor.
    ///
    /// Execute FETCH LAST (same as ABSOLUTE -1)
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_last<'a>(slf: Py<Self>) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(format!("FETCH LAST FROM {cursor_name}"), None, Some(false))
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch absolute row from cursor.
    ///
    /// Execute FETCH ABSOLUTE<absolute_number>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_absolute<'a>(
        slf: Py<Self>,
        absolute_number: i64,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(
                    format!("FETCH ABSOLUTE {absolute_number} FROM {cursor_name}"),
                    None,
                    Some(false),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch absolute row from cursor.
    ///
    /// Execute FETCH ABSOLUTE<absolute_number>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_relative<'a>(
        slf: Py<Self>,
        relative_number: i64,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(
                    format!("FETCH  RELATIVE {relative_number} FROM {cursor_name}"),
                    None,
                    Some(false),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch forward all from cursor.
    ///
    /// Execute FORWARD ALL.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_forward_all<'a>(
        slf: Py<Self>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(
                    format!("FETCH FORWARD ALL FROM {cursor_name}"),
                    None,
                    Some(false),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch backward from cursor.
    ///
    /// Execute BACKWARD <backward_count>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_backward<'a>(
        slf: Py<Self>,
        backward_count: i64,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(
                    format!("FETCH BACKWARD {backward_count} FROM {cursor_name}",),
                    None,
                    Some(false),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }

    /// Fetch backward from cursor.
    ///
    /// Execute BACKWARD <backward_count>.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn fetch_backward_all<'a>(
        slf: Py<Self>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let (db_transaction, cursor_name) = Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (self_.db_transaction.clone(), self_.cursor_name.clone())
        });

        if let Some(db_transaction) = db_transaction {
            let result = db_transaction
                .psqlpy_query(
                    format!("FETCH BACKWARD ALL FROM {cursor_name}"),
                    None,
                    Some(false),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::CursorFetchError(format!(
                        "Cannot fetch data from cursor, error - {err}"
                    ))
                })?;
            return Ok(result);
        }

        Err(RustPSQLDriverError::CursorClosedError)
    }
}
