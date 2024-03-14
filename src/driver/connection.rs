use deadpool_postgres::Object;
use pyo3::{pyclass, pymethods, PyAny, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::types::ToSql;

use crate::{
    common::rustengine_future,
    exceptions::rust_errors::RustPSQLDriverPyResult,
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO},
};

use super::{
    transaction::{RustTransaction, Transaction},
    transaction_options::{IsolationLevel, ReadVariant},
};

#[allow(clippy::module_name_repetitions)]
pub struct RustConnection {
    pub db_client: Arc<Object>,
}

impl RustConnection {
    #[must_use]
    pub fn new(db_client: Arc<Object>) -> Self {
        RustConnection { db_client }
    }
    /// Execute statement with or witout parameters.
    ///
    /// # Errors
    ///
    /// May return Err Result if
    /// 1) Cannot convert incoming parameters
    /// 2) Cannot prepare statement
    /// 3) Cannot execute query
    pub async fn inner_execute(
        &self,
        querystring: String,
        params: Vec<PythonDTO>,
        prepared: bool,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client = &self.db_client;
        let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(params.len());
        for param in &params {
            vec_parameters.push(param);
        }

        let result = if prepared {
            db_client
                .query(
                    &db_client.prepare_cached(&querystring).await?,
                    &vec_parameters.into_boxed_slice(),
                )
                .await?
        } else {
            db_client
                .query(&querystring, &vec_parameters.into_boxed_slice())
                .await?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Return new instance of transaction.
    #[must_use]
    pub fn inner_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> Transaction {
        let inner_transaction = RustTransaction::new(
            self.db_client.clone(),
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

#[pyclass()]
pub struct Connection {
    pub inner_connection: Arc<RustConnection>,
}

impl Connection {
    #[must_use]
    pub fn new(inner_connection: Arc<RustConnection>) -> Self {
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
        rustengine_future(py, async move {
            connection_arc
                .inner_execute(querystring, params, prepared.unwrap_or(true))
                .await
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
        self.inner_connection
            .inner_transaction(isolation_level, read_variant, deferrable)
    }
}
