use deadpool_postgres::Object;
use pyo3::{pyclass, pymethods, PyAny, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::types::ToSql;

use crate::{
    common::rustdriver_future,
    exceptions::rust_errors::RustPSQLDriverPyResult,
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO},
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
        parameters: Option<&'a PyAny>,
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
