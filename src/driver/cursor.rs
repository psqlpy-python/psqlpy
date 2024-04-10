// use pyo3::{
//     exceptions::PyStopAsyncIteration, pyclass, pymethods, Py, PyAny, PyErr, PyObject, PyRef,
//     PyRefMut, Python,
// };
// use std::sync::Arc;
// use tokio_postgres::Row;

// use crate::{
//     common::rustdriver_future,
//     exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
//     query_result::PSQLDriverPyQueryResult,
//     value_converter::PythonDTO,
// };

// use super::transaction::RustTransaction;

// #[allow(clippy::module_name_repetitions)]
// pub struct InnerCursor {
//     querystring: String,
//     parameters: Vec<PythonDTO>,
//     db_transaction: Arc<tokio::sync::RwLock<RustTransaction>>,
//     cursor_name: String,
//     fetch_number: usize,
//     scroll: Option<bool>,
//     prepared: bool,
//     is_started: bool,
//     closed: bool,
// }

// impl InnerCursor {
//     #[must_use]
//     pub fn new(
//         db_transaction: Arc<tokio::sync::RwLock<RustTransaction>>,
//         querystring: String,
//         parameters: Vec<PythonDTO>,
//         cursor_name: String,
//         scroll: Option<bool>,
//         fetch_number: usize,
//         prepared: bool,
//     ) -> Self {
//         InnerCursor {
//             querystring,
//             parameters,
//             db_transaction,
//             cursor_name,
//             fetch_number,
//             scroll,
//             prepared,
//             is_started: false,
//             closed: false,
//         }
//     }

//     /// Start the cursor.
//     ///
//     /// It executes DECLARE command to create cursor in the transaction.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub async fn inner_start(&mut self) -> RustPSQLDriverPyResult<()> {
//         let db_transaction_arc = self.db_transaction.clone();

//         let mut cursor_init_query = format!("DECLARE {}", self.cursor_name);
//         if let Some(scroll) = self.scroll {
//             if scroll {
//                 cursor_init_query.push_str(" SCROLL");
//             } else {
//                 cursor_init_query.push_str(" NO SCROLL");
//             }
//         }

//         cursor_init_query.push_str(format!(" CURSOR FOR {}", self.querystring).as_str());

//         db_transaction_arc
//             .read()
//             .await
//             .inner_execute(cursor_init_query, self.parameters.clone(), self.prepared)
//             .await?;

//         self.is_started = true;
//         Ok(())
//     }

//     /// Close the cursor.
//     ///
//     /// It executes CLOSE command to close cursor in the transaction.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub async fn inner_close(&mut self) -> RustPSQLDriverPyResult<()> {
//         let db_transaction_arc = self.db_transaction.clone();

//         if self.closed {
//             return Err(RustPSQLDriverError::DataBaseCursorError(
//                 "Cursor is already closed".into(),
//             ));
//         }

//         db_transaction_arc
//             .read()
//             .await
//             .inner_execute(format!("CLOSE {}", self.cursor_name), vec![], false)
//             .await?;

//         self.closed = true;
//         Ok(())
//     }

//     /// Execute querystring for cursor.
//     ///
//     /// This is the main method for executing any cursor querystring.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub async fn inner_execute(
//         &self,
//         querystring: String,
//         prepared: bool,
//     ) -> RustPSQLDriverPyResult<Vec<Row>> {
//         let db_transaction_arc = self.db_transaction.clone();

//         if !self.is_started {
//             return Err(RustPSQLDriverError::DataBaseCursorError(
//                 "Cursor is not opened, please call `start()`.".into(),
//             ));
//         }

//         let result = db_transaction_arc
//             .read()
//             .await
//             .inner_execute_raw(querystring, vec![], prepared)
//             .await?;

//         Ok(result)
//     }
// }

// #[pyclass]
// pub struct Cursor {
//     inner_cursor: Arc<tokio::sync::RwLock<InnerCursor>>,
// }

// impl Cursor {
//     #[must_use]
//     pub fn new(inner_cursor: InnerCursor) -> Self {
//         Cursor {
//             inner_cursor: Arc::new(tokio::sync::RwLock::new(inner_cursor)),
//         }
//     }
// }

// #[pymethods]
// impl Cursor {
//     #[allow(clippy::missing_errors_doc)]
//     #[allow(clippy::needless_pass_by_value)]
//     pub fn __await__<'a>(
//         slf: PyRefMut<'a, Self>,
//         _py: Python,
//     ) -> RustPSQLDriverPyResult<PyRefMut<'a, Self>> {
//         Ok(slf)
//     }

//     #[allow(clippy::needless_pass_by_value)]
//     fn __aenter__<'a>(
//         slf: PyRefMut<'a, Self>,
//         py: Python<'a>,
//     ) -> RustPSQLDriverPyResult<&'a PyAny> {
//         let inner_cursor_arc = slf.inner_cursor.clone();
//         let inner_cursor_arc2 = slf.inner_cursor.clone();
//         rustdriver_future(py, async move {
//             let mut inner_cursor_guard = inner_cursor_arc.write().await;
//             inner_cursor_guard.inner_start().await?;
//             Ok(Cursor {
//                 inner_cursor: inner_cursor_arc2,
//             })
//         })
//     }

//     #[allow(clippy::needless_pass_by_value)]
//     fn __aexit__<'a>(
//         slf: PyRefMut<'a, Self>,
//         py: Python<'a>,
//         _exception_type: Py<PyAny>,
//         exception: &PyAny,
//         _traceback: Py<PyAny>,
//     ) -> RustPSQLDriverPyResult<&'a PyAny> {
//         let inner_cursor_arc = slf.inner_cursor.clone();
//         let inner_cursor_arc2 = slf.inner_cursor.clone();
//         let is_no_exc = exception.is_none();
//         let py_err = PyErr::from_value(exception);

//         rustdriver_future(py, async move {
//             let mut inner_cursor_guard = inner_cursor_arc.write().await;
//             if is_no_exc {
//                 inner_cursor_guard.inner_close().await?;
//                 Ok(Cursor {
//                     inner_cursor: inner_cursor_arc2,
//                 })
//             } else {
//                 inner_cursor_guard.inner_close().await?;
//                 Err(RustPSQLDriverError::PyError(py_err))
//             }
//         })
//     }

//     #[must_use]
//     pub fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
//         slf
//     }

//     /// Return next result from the SQL statement.
//     ///
//     /// Execute FETCH <number> FROM <cursor name>
//     ///
//     /// # Errors
//     /// May return Err Result if can't execute querystring.
//     pub fn __anext__(&self, py: Python<'_>) -> RustPSQLDriverPyResult<Option<PyObject>> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         let future = rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!(
//                         "FETCH {} FROM {}",
//                         inner_cursor_guard.fetch_number, inner_cursor_guard.cursor_name,
//                     ),
//                     false,
//                 )
//                 .await?;

//             if result.is_empty() {
//                 return Err(PyStopAsyncIteration::new_err(
//                     "Iteration is over, no more results in cursor",
//                 )
//                 .into());
//             };

//             Ok(PSQLDriverPyQueryResult::new(result))
//         });

//         Ok(Some(future?.into()))
//     }

//     /// Start the cursor.
//     ///
//     /// It executes DECLARE command to create cursor in the transaction.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn start<'a>(&'a mut self, py: Python<'a>) -> RustPSQLDriverPyResult<&'a PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let mut inner_cursor_guard = inner_cursor_arc.write().await;
//             inner_cursor_guard.inner_start().await
//         })
//     }

//     /// Close cursor.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute CLOSE command
//     pub fn close<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let mut inner_cursor_guard = inner_cursor_arc.write().await;
//             inner_cursor_guard.inner_close().await
//         })
//     }

//     /// Fetch data from cursor.
//     ///
//     /// It's possible to specify fetch number.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch<'a>(
//         &'a self,
//         py: Python<'a>,
//         fetch_number: Option<usize>,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let fetch_number = match fetch_number {
//                 Some(usize) => usize,
//                 None => inner_cursor_guard.fetch_number,
//             };
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!(
//                         "FETCH {fetch_number} FROM {}",
//                         inner_cursor_guard.cursor_name
//                     ),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch row from cursor.
//     ///
//     /// Execute FETCH NEXT.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_next<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!("FETCH NEXT FROM {}", inner_cursor_guard.cursor_name),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch previous from cursor.
//     ///
//     /// Execute FETCH PRIOR.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_prior<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!("FETCH PRIOR FROM {}", inner_cursor_guard.cursor_name),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch first row from cursor.
//     ///
//     /// Execute FETCH FIRST (same as ABSOLUTE 1)
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_first<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!("FETCH FIRST FROM {}", inner_cursor_guard.cursor_name),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch last row from cursor.
//     ///
//     /// Execute FETCH LAST (same as ABSOLUTE -1)
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_last<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!("FETCH LAST FROM {}", inner_cursor_guard.cursor_name),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch absolute row from cursor.
//     ///
//     /// Execute FETCH ABSOLUTE<absolute_number>.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_absolute<'a>(
//         &'a self,
//         py: Python<'a>,
//         absolute_number: i64,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!(
//                         "FETCH ABSOLUTE {absolute_number} FROM {}",
//                         inner_cursor_guard.cursor_name
//                     ),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch relative row from cursor.
//     ///
//     /// Execute FETCH RELATIVE<absolute_number>.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_relative<'a>(
//         &'a self,
//         py: Python<'a>,
//         relative_number: i64,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!(
//                         "FETCH RELATIVE {relative_number} FROM {}",
//                         inner_cursor_guard.cursor_name
//                     ),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch forward all from cursor.
//     ///
//     /// Execute FORWARD ALL.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_forward_all<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!("FETCH FORWARD ALL FROM {}", inner_cursor_guard.cursor_name),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch backward from cursor.
//     ///
//     /// Execute BACKWARD <backward_count>.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_backward<'a>(
//         &'a self,
//         py: Python<'a>,
//         backward_count: i64,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!(
//                         "FETCH BACKWARD {backward_count} FROM {}",
//                         inner_cursor_guard.cursor_name
//                     ),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }

//     /// Fetch backward from cursor.
//     ///
//     /// Execute BACKWARD <backward_count>.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot execute query.
//     pub fn fetch_backward_all<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&PyAny> {
//         let inner_cursor_arc = self.inner_cursor.clone();

//         rustdriver_future(py, async move {
//             let inner_cursor_guard = inner_cursor_arc.read().await;
//             let result = inner_cursor_guard
//                 .inner_execute(
//                     format!("FETCH BACKWARD ALL FROM {}", inner_cursor_guard.cursor_name),
//                     false,
//                 )
//                 .await?;
//             Ok(PSQLDriverPyQueryResult::new(result))
//         })
//     }
// }

use std::sync::Arc;

use deadpool_postgres::Object;
use pyo3::{
    exceptions::PyStopAsyncIteration, pyclass, pymethods, Py, PyAny, PyErr, PyObject, Python,
};

use crate::runtime::rustdriver_future;
use crate::{
    common::ObjectQueryTrait,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
};

trait CursorObjectTrait {
    async fn cursor_start(
        &self,
        cursor_name: &String,
        scroll: &Option<bool>,
        querystring: &String,
        prepared: &Option<bool>,
        parameters: &Option<Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<()>;

    async fn cursor_close(&self, closed: &bool, cursor_name: &String)
        -> RustPSQLDriverPyResult<()>;
}

impl CursorObjectTrait for Object {
    async fn cursor_start(
        &self,
        cursor_name: &String,
        scroll: &Option<bool>,
        querystring: &String,
        prepared: &Option<bool>,
        parameters: &Option<Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<()> {
        let mut cursor_init_query = format!("DECLARE {}", cursor_name);
        if let Some(scroll) = scroll {
            if *scroll {
                cursor_init_query.push_str(" SCROLL");
            } else {
                cursor_init_query.push_str(" NO SCROLL");
            }
        }

        cursor_init_query.push_str(format!(" CURSOR FOR {}", querystring).as_str());

        self.psqlpy_query(cursor_init_query, parameters.clone(), *prepared)
            .await?;

        Ok(())
    }

    async fn cursor_close(
        &self,
        closed: &bool,
        cursor_name: &String,
    ) -> RustPSQLDriverPyResult<()> {
        if *closed {
            return Err(RustPSQLDriverError::DataBaseCursorError(
                "Cursor is already closed".into(),
            ));
        }

        self.psqlpy_query(
            format!("CLOSE {}", cursor_name),
            Default::default(),
            Some(false),
        )
        .await?;

        Ok(())
    }
}

#[pyclass]
pub struct Cursor {
    db_transaction: Arc<Object>,
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
            db_transaction,
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
                return (
                    self_.db_transaction.clone(),
                    self_.cursor_name.clone(),
                    self_.scroll,
                    self_.querystring.clone(),
                    self_.prepared,
                    self_.parameters.clone(),
                );
            });
        db_transaction
            .cursor_start(&cursor_name, &scroll, &querystring, &prepared, &parameters)
            .await?;
        Python::with_gil(|gil| {
            let mut self_ = slf.borrow_mut(gil);
            self_.is_started = true;
        });
        Ok(slf)
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

        db_transaction.cursor_close(&closed, &cursor_name).await?;
        if !is_exception_none {
            return Err(RustPSQLDriverError::PyError(py_err));
        }
        Ok(())
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
            let future = rustdriver_future(gil, async move {
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

                Ok(result)
            });

            future
        });

        Ok(Some(py_future?.into()))
    }

    pub async fn start(&mut self) -> RustPSQLDriverPyResult<()> {
        let db_transaction_arc = self.db_transaction.clone();

        db_transaction_arc
            .cursor_start(
                &self.cursor_name,
                &self.scroll,
                &self.querystring,
                &self.prepared,
                &self.parameters,
            )
            .await?;

        self.is_started = true;
        Ok(())
    }

    /// Close the cursor.
    ///
    /// It executes CLOSE command to close cursor in the transaction.
    ///
    /// # Errors
    /// May return Err Result if cannot execute query.
    pub async fn close(&mut self) -> RustPSQLDriverPyResult<()> {
        let db_transaction_arc = self.db_transaction.clone();

        db_transaction_arc
            .cursor_close(&self.closed, &self.cursor_name)
            .await?;

        self.closed = true;
        Ok(())
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
            return (
                self_.db_transaction.clone(),
                self_.fetch_number,
                self_.cursor_name.clone(),
            );
        });

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
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(format!("FETCH NEXT FROM {cursor_name}"), None, Some(false))
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(format!("FETCH PRIOR FROM {cursor_name}"), None, Some(false))
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(format!("FETCH FIRST FROM {cursor_name}"), None, Some(false))
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(format!("FETCH LAST FROM {cursor_name}"), None, Some(false))
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(
                format!("FETCH ABSOLUTE {absolute_number} FROM {cursor_name}"),
                None,
                Some(false),
            )
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(
                format!("FETCH  RELATIVE {relative_number} FROM {cursor_name}"),
                None,
                Some(false),
            )
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(
                format!("FETCH FORWARD ALL FROM {cursor_name}"),
                None,
                Some(false),
            )
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(
                format!("FETCH BACKWARD {backward_count} FROM {cursor_name}",),
                None,
                Some(false),
            )
            .await?;
        Ok(result)
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
            return (self_.db_transaction.clone(), self_.cursor_name.clone());
        });

        let result = db_transaction
            .psqlpy_query(
                format!("FETCH BACKWARD ALL FROM {cursor_name}"),
                None,
                Some(false),
            )
            .await?;
        Ok(result)
    }
}
