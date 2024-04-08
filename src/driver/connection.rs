use deadpool_postgres::Object;
use pyo3::{pyclass, pymethods, Py, PyAny, Python};
use std::{sync::Arc, vec};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, postgres_to_py, PythonDTO, QueryParameter},
};

// #[allow(clippy::module_name_repetitions)]
// pub struct RustConnection {
//     pub db_client: Arc<tokio::sync::RwLock<Object>>,
// }

// impl RustConnection {
//     #[must_use]
//     pub fn new(db_client: Arc<tokio::sync::RwLock<Object>>) -> Self {
//         RustConnection { db_client }
//     }
//     /// Execute statement with or witout parameters.
//     ///
//     /// # Errors
//     ///
//     /// May return Err Result if
//     /// 1) Cannot convert incoming parameters
//     /// 2) Cannot prepare statement
//     /// 3) Cannot execute query
//     pub async fn inner_execute(
//         &self,
//         querystring: String,
//         parameters: Vec<PythonDTO>,
//         prepared: bool,
//     ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
//         let db_client = self.db_client.read().await;
//         let vec_parameters: Vec<&QueryParameter> = parameters
//             .iter()
//             .map(|param| param as &QueryParameter)
//             .collect();

//         let result = if prepared {
//             db_client
//                 .query(
//                     &db_client.prepare_cached(&querystring).await?,
//                     &vec_parameters.into_boxed_slice(),
//                 )
//                 .await?
//         } else {
//             db_client
//                 .query(&querystring, &vec_parameters.into_boxed_slice())
//                 .await?
//         };

//         Ok(PSQLDriverPyQueryResult::new(result))
//     }

//     /// Execute querystring with many parameters.
//     ///
//     /// Method doesn't acquire lock on any structure fields.
//     /// It prepares and caches querystring in the inner Object object.
//     ///
//     /// Then execute the query.
//     ///
//     /// # Errors
//     /// May return Err Result if:
//     /// 1) Transaction is not started
//     /// 2) Transaction is done already
//     /// 3) Can not create/retrieve prepared statement
//     /// 4) Can not execute statement
//     pub async fn inner_execute_many(
//         &self,
//         querystring: String,
//         parameters: Vec<Vec<PythonDTO>>,
//         prepared: bool,
//     ) -> RustPSQLDriverPyResult<()> {
//         let mut db_client = self.db_client.write().await;
//         let transaction = db_client.transaction().await?;
//         for single_parameters in parameters {
//             if prepared {
//                 transaction
//                     .query(
//                         &transaction.prepare_cached(&querystring).await?,
//                         &single_parameters
//                             .iter()
//                             .map(|p| p as &QueryParameter)
//                             .collect::<Vec<_>>(),
//                     )
//                     .await?;
//             } else {
//                 transaction
//                     .query(
//                         &querystring,
//                         &single_parameters
//                             .iter()
//                             .map(|p| p as &QueryParameter)
//                             .collect::<Vec<_>>(),
//                     )
//                     .await?;
//             }
//         }

//         transaction.commit().await?;

//         Ok(())
//     }

//     /// Fetch exaclty single row from query.
//     ///
//     /// Method doesn't acquire lock on any structure fields.
//     /// It prepares and caches querystring in the inner Object object.
//     ///
//     /// Then execute the query.
//     ///
//     /// # Errors
//     /// May return Err Result if:
//     /// 1) Transaction is not started
//     /// 2) Transaction is done already
//     /// 3) Can not create/retrieve prepared statement
//     /// 4) Can not execute statement
//     /// 5) Query returns more than one row
//     pub async fn inner_fetch_row(
//         &self,
//         querystring: String,
//         parameters: Vec<PythonDTO>,
//         prepared: bool,
//     ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
//         let vec_parameters: Vec<&QueryParameter> = parameters
//             .iter()
//             .map(|param| param as &QueryParameter)
//             .collect();
//         let db_client_guard = self.db_client.read().await;

//         let result = if prepared {
//             db_client_guard
//                 .query_one(
//                     &db_client_guard.prepare_cached(&querystring).await?,
//                     &vec_parameters.into_boxed_slice(),
//                 )
//                 .await?
//         } else {
//             db_client_guard
//                 .query_one(&querystring, &vec_parameters.into_boxed_slice())
//                 .await?
//         };

//         Ok(PSQLDriverSinglePyQueryResult::new(result))
//     }

//     /// Execute querystring with parameters.
//     ///
//     /// Method doesn't acquire lock on any structure fields.
//     /// It prepares and caches querystring in the inner Object object.
//     ///
//     /// Then execute the query.
//     ///
//     /// It returns `Vec<Row>` instead of `PSQLDriverPyQueryResult`.
//     ///
//     /// # Errors
//     /// May return Err Result if:
//     /// 1) Transaction is not started
//     /// 2) Transaction is done already
//     /// 3) Can not create/retrieve prepared statement
//     /// 4) Can not execute statement
//     pub async fn inner_execute_raw(
//         &self,
//         querystring: String,
//         parameters: Vec<PythonDTO>,
//         prepared: bool,
//     ) -> RustPSQLDriverPyResult<Vec<Row>> {
//         let db_client_guard = self.db_client.read().await;
//         let vec_parameters: Vec<&QueryParameter> = parameters
//             .iter()
//             .map(|param| param as &QueryParameter)
//             .collect();

//         let result = if prepared {
//             db_client_guard
//                 .query(
//                     &db_client_guard.prepare_cached(&querystring).await?,
//                     &vec_parameters.into_boxed_slice(),
//                 )
//                 .await?
//         } else {
//             db_client_guard
//                 .query(&querystring, &vec_parameters.into_boxed_slice())
//                 .await?
//         };

//         Ok(result)
//     }

//     /// Return new instance of transaction.
//     #[must_use]
//     pub fn inner_transaction(
//         &self,
//         isolation_level: Option<IsolationLevel>,
//         read_variant: Option<ReadVariant>,
//         deferrable: Option<bool>,
//     ) -> Transaction {
//         let inner_transaction = RustTransaction::new(
//             Arc::new(RustConnection::new(self.db_client.clone())),
//             false,
//             false,
//             Arc::new(tokio::sync::RwLock::new(HashSet::new())),
//             isolation_level,
//             read_variant,
//             deferrable,
//         );

//         Transaction::new(
//             Arc::new(tokio::sync::RwLock::new(inner_transaction)),
//             Default::default(),
//         )
//     }
// }

// #[pyclass()]
// pub struct Connection {
//     pub inner_connection: Arc<RustConnection>,
// }

// impl Connection {
//     #[must_use]
//     pub fn new(inner_connection: Arc<RustConnection>) -> Self {
//         Connection { inner_connection }
//     }
// }

// #[pymethods]
// impl Connection {
//     /// Execute statement with or witout parameters.
//     ///
//     /// # Errors
//     ///
//     /// May return Err Result if
//     /// 1) Cannot convert incoming parameters
//     /// 2) Cannot prepare statement
//     /// 3) Cannot execute query
//     pub fn execute<'a>(
//         &'a self,
//         py: Python<'a>,
//         querystring: String,
//         parameters: Option<&PyAny>,
//         prepared: Option<bool>,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let mut params: Vec<PythonDTO> = vec![];
//         if let Some(parameters) = parameters {
//             params = convert_parameters(parameters)?;
//         }
//         let inner_connection_arc = self.inner_connection.clone();
//         rustdriver_future(py, async move {
//             inner_connection_arc
//                 .inner_execute(querystring, params, prepared.unwrap_or(true))
//                 .await
//         })
//     }

//     /// Execute querystring with parameters.
//     ///
//     /// It converts incoming parameters to rust readable
//     /// and then execute the query with them.
//     ///
//     /// # Errors
//     ///
//     /// May return Err Result if:
//     /// 1) Cannot convert python parameters
//     /// 2) Cannot execute querystring.
//     pub fn execute_many<'a>(
//         &'a self,
//         py: Python<'a>,
//         querystring: String,
//         parameters: Option<&'a PyList>,
//         prepared: Option<bool>,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let transaction_arc = self.inner_connection.clone();
//         let mut params: Vec<Vec<PythonDTO>> = vec![];
//         if let Some(parameters) = parameters {
//             for single_parameters in parameters {
//                 params.push(convert_parameters(single_parameters)?);
//             }
//         }

//         rustdriver_future(py, async move {
//             transaction_arc
//                 .inner_execute_many(querystring, params, prepared.unwrap_or(true))
//                 .await
//         })
//     }

// / Execute querystring with parameters and return first row.
// /
// / It converts incoming parameters to rust readable,
// / executes query with them and returns first row of response.
// /
// / # Errors
// /
// / May return Err Result if:
// / 1) Cannot convert python parameters
// / 2) Cannot execute querystring.
// / 3) Query returns more than one row.
// pub fn fetch_row<'a>(
//     &'a self,
//     py: Python<'a>,
//     querystring: String,
//     parameters: Option<&'a PyList>,
//     prepared: Option<bool>,
// ) -> RustPSQLDriverPyResult<&PyAny> {
//     let transaction_arc = self.inner_connection.clone();
//     let mut params: Vec<PythonDTO> = vec![];
//     if let Some(parameters) = parameters {
//         params = convert_parameters(parameters)?;
//     }

//     rustdriver_future(py, async move {
//         transaction_arc
//             .inner_fetch_row(querystring, params, prepared.unwrap_or(true))
//             .await
//     })
// }

//     /// Execute querystring with parameters and return first value in the first row.
//     ///
//     /// It converts incoming parameters to rust readable,
//     /// executes query with them and returns first row of response.
//     ///
//     /// # Errors
//     ///
//     /// May return Err Result if:
//     /// 1) Cannot convert python parameters
//     /// 2) Cannot execute querystring.
//     /// 3) Query returns more than one row
//     pub fn fetch_val<'a>(
//         &'a self,
//         py: Python<'a>,
//         querystring: String,
//         parameters: Option<&'a PyList>,
//         prepared: Option<bool>,
//     ) -> RustPSQLDriverPyResult<&PyAny> {
//         let transaction_arc = self.inner_connection.clone();
//         let mut params: Vec<PythonDTO> = vec![];
//         if let Some(parameters) = parameters {
//             params = convert_parameters(parameters)?;
//         }

//         rustdriver_future(py, async move {
//             let first_row = transaction_arc
//                 .inner_fetch_row(querystring, params, prepared.unwrap_or(true))
//                 .await?
//                 .get_inner();
//             Python::with_gil(|py| match first_row.columns().first() {
//                 Some(first_column) => postgres_to_py(py, &first_row, first_column, 0),
//                 None => Ok(py.None()),
//             })
//         })
//     }

//     /// Return new instance of transaction.
//     #[must_use]
//     pub fn transaction(
//         &self,
//         isolation_level: Option<IsolationLevel>,
//         read_variant: Option<ReadVariant>,
//         deferrable: Option<bool>,
//     ) -> Transaction {
//         let inner_transaction = RustTransaction::new(
//             self.inner_connection.clone(),
//             false,
//             false,
//             Arc::new(tokio::sync::RwLock::new(HashSet::new())),
//             isolation_level,
//             read_variant,
//             deferrable,
//         );

//         Transaction::new(
//             Arc::new(tokio::sync::RwLock::new(inner_transaction)),
//             Default::default(),
//         )
//     }
// }

#[pyclass]
pub struct Connection {
    pub db_client: Arc<Object>,
}

impl Connection {
    pub fn new(db_client: Object) -> Self {
        Connection {
            db_client: Arc::new(db_client),
        }
    }
}

#[pymethods]
impl Connection {
    /// Execute statement with or witout parameters.
    ///
    /// # Errors
    ///
    /// May return Err Result if
    /// 1) Cannot convert incoming parameters
    /// 2) Cannot prepare statement
    /// 3) Cannot execute query
    pub async fn execute(
        self_: pyo3::Py<Self>,
        querystring: String,
        prepared: Option<bool>,
        parameters: Option<pyo3::Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query(
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
                .query(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
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
    pub async fn execute_many<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        prepared: Option<bool>,
        parameters: Option<Vec<Py<PyAny>>>,
    ) -> RustPSQLDriverPyResult<()> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());
        let mut params: Vec<Vec<PythonDTO>> = vec![];
        if let Some(parameters) = parameters {
            for vec_of_py_any in parameters {
                params.push(convert_parameters(vec_of_py_any)?);
            }
        }
        let prepared = prepared.unwrap_or(true);

        db_client.batch_execute("BEGIN;").await.map_err(|err| {
            RustPSQLDriverError::DataBaseTransactionError(format!(
                "Cannot start transaction to run execute_many: {err}"
            ))
        })?;
        for param in params {
            let querystring_result = if prepared {
                let prepared_stmt = &db_client.prepare_cached(&querystring).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::DataBaseTransactionError(format!(
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

            if let Err(error) = querystring_result {
                db_client.batch_execute("ROLLBACK;").await?;
                return Err(RustPSQLDriverError::DataBaseTransactionError(format!(
                    "Error occured in `execute_many` statement, transaction is rolled back: {error}"
                )));
            }
        }

        db_client.batch_execute("COMMIT;").await?;

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
    pub async fn fetch_row(
        self_: pyo3::Py<Self>,
        querystring: String,
        prepared: Option<bool>,
        parameters: Option<pyo3::Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
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

        Ok(PSQLDriverSinglePyQueryResult::new(result))
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
    pub async fn fetch_val<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        prepared: Option<bool>,
        parameters: Option<pyo3::Py<PyAny>>,
    ) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
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

        Python::with_gil(|gil| match result.columns().first() {
            Some(first_column) => postgres_to_py(gil, &result, first_column, 0),
            None => Ok(gil.None()),
        })
    }
}
