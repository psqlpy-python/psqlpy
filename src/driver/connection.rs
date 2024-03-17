use deadpool_postgres::Object;
use pyo3::{pyclass, pymethods, types::PyList, PyAny, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::types::ToSql;

use crate::{
    common::rustdriver_future,
    exceptions::rust_errors::RustPSQLDriverPyResult,
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, postgres_to_py, PythonDTO},
};

use super::{
    transaction::{RustTransaction, Transaction},
    transaction_options::{IsolationLevel, ReadVariant},
};

#[pyclass()]
pub struct Connection {
    pub inner_connection: Arc<Object>,
}

impl Connection {
    #[must_use]
    pub fn new(inner_connection: Arc<Object>) -> Self {
        Connection { inner_connection }
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
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&PyAny>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let connection_arc = self.inner_connection.clone();

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let is_prepared = prepared.unwrap_or(true);
        rustdriver_future(py, async move {
            let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(params.len());
            for param in &params {
                vec_parameters.push(param);
            }

            let result = if is_prepared {
                connection_arc
                    .query(
                        &connection_arc.prepare_cached(&querystring).await?,
                        &vec_parameters.into_boxed_slice(),
                    )
                    .await?
            } else {
                connection_arc
                    .query(&querystring, &vec_parameters.into_boxed_slice())
                    .await?
            };

            Ok(PSQLDriverPyQueryResult::new(result))
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
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let connection_arc = self.inner_connection.clone();
        let mut params: Vec<Vec<PythonDTO>> = vec![];
        if let Some(parameters) = parameters {
            for single_parameters in parameters {
                params.push(convert_parameters(single_parameters)?);
            }
        }

        let is_prepared = prepared.unwrap_or(true);
        rustdriver_future(py, async move {
            for single_parameters in params {
                if is_prepared {
                    connection_arc
                        .query(
                            &connection_arc.prepare_cached(&querystring).await?,
                            &single_parameters
                                .iter()
                                .map(|p| p as &(dyn ToSql + Sync))
                                .collect::<Vec<_>>(),
                        )
                        .await?;
                } else {
                    connection_arc
                        .query(
                            &querystring,
                            &single_parameters
                                .iter()
                                .map(|p| p as &(dyn ToSql + Sync))
                                .collect::<Vec<_>>(),
                        )
                        .await?;
                }
            }

            Ok(())
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
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let connection_arc = self.inner_connection.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }

        rustdriver_future(py, async move {
            let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(params.len());
            for param in &params {
                vec_parameters.push(param);
            }

            let result = if prepared.unwrap_or(true) {
                connection_arc
                    .query_one(
                        &connection_arc.prepare_cached(&querystring).await?,
                        &vec_parameters.into_boxed_slice(),
                    )
                    .await?
            } else {
                connection_arc
                    .query_one(&querystring, &vec_parameters.into_boxed_slice())
                    .await?
            };

            Ok(PSQLDriverSinglePyQueryResult::new(result))
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
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let connection_arc = self.inner_connection.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }

        rustdriver_future(py, async move {
            let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(params.len());
            for param in &params {
                vec_parameters.push(param);
            }

            let first_row = if prepared.unwrap_or(true) {
                connection_arc
                    .query_one(
                        &connection_arc.prepare_cached(&querystring).await?,
                        &vec_parameters.into_boxed_slice(),
                    )
                    .await?
            } else {
                connection_arc
                    .query_one(&querystring, &vec_parameters.into_boxed_slice())
                    .await?
            };
            Python::with_gil(|py| match first_row.columns().first() {
                Some(first_column) => postgres_to_py(py, &first_row, first_column, 0),
                None => Ok(py.None()),
            })
        })
    }

    /// Return new instance of transaction.
    #[must_use]
    pub fn transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> Transaction {
        let inner_transaction = RustTransaction::new(
            self.inner_connection.clone(),
            false,
            false,
            Arc::new(tokio::sync::RwLock::new(HashSet::new())),
            isolation_level,
            read_variant,
            deferrable,
        );

        Transaction::new(
            Arc::new(tokio::sync::RwLock::new(inner_transaction)),
            Default::default(),
        )
    }
}
