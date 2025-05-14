// use std::sync::Arc;

// use pyo3::{pyclass, pymethods, Py, PyObject, Python};
// use tokio_postgres::Portal as tp_Portal;

// use crate::{
//     exceptions::rust_errors::PSQLPyResult, query_result::PSQLDriverPyQueryResult,
//     runtime::rustdriver_future,
// };

// use super::inner_transaction::PsqlpyTransaction;

// #[pyclass]
// pub struct Portal {
//     transaction: Arc<PsqlpyTransaction>,
//     inner: tp_Portal,
//     array_size: i32,
// }

// impl Portal {
//     pub fn new(transaction: Arc<PsqlpyTransaction>, inner: tp_Portal, array_size: i32) -> Self {
//         Self {
//             transaction,
//             inner,
//             array_size,
//         }
//     }

//     async fn query_portal(&self, size: i32) -> PSQLPyResult<PSQLDriverPyQueryResult> {
//         let result = self.transaction.query_portal(&self.inner, size).await?;
//         Ok(PSQLDriverPyQueryResult::new(result))
//     }
// }

// #[pymethods]
// impl Portal {
//     #[getter]
//     fn get_array_size(&self) -> i32 {
//         self.array_size
//     }

//     #[setter]
//     fn set_array_size(&mut self, value: i32) {
//         self.array_size = value;
//     }

//     fn __aiter__(slf: Py<Self>) -> Py<Self> {
//         slf
//     }

//     fn __await__(slf: Py<Self>) -> Py<Self> {
//         slf
//     }

//     fn __anext__(&self) -> PSQLPyResult<Option<PyObject>> {
//         let transaction = self.transaction.clone();
//         let portal = self.inner.clone();
//         let size = self.array_size.clone();

//         let py_future = Python::with_gil(move |gil| {
//             rustdriver_future(gil, async move {
//                 let result = transaction.query_portal(&portal, size).await?;

//                 Ok(PSQLDriverPyQueryResult::new(result))
//             })
//         });

//         Ok(Some(py_future?))
//     }

//     async fn fetch_one(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
//         self.query_portal(1).await
//     }

//     #[pyo3(signature = (size=None))]
//     async fn fetch_many(&self, size: Option<i32>) -> PSQLPyResult<PSQLDriverPyQueryResult> {
//         self.query_portal(size.unwrap_or(self.array_size)).await
//     }

//     async fn fetch_all(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
//         self.query_portal(-1).await
//     }

//     async fn close(&mut self) {
//         let _ = Arc::downgrade(&self.transaction);
//     }
// }
