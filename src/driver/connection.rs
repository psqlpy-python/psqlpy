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
    transaction_options::IsolationLevel,
};

pub struct RustConnection {
    pub db_client: Arc<tokio::sync::RwLock<Object>>,
}

impl RustConnection {
    /// Execute querystring with parameters.
    ///
    /// Method doesn't acquire lock on database connection.
    /// It prepares and caches querystring in the inner Object object.
    ///
    /// Then execute the query.
    ///
    /// # Errors:
    /// May return Err Result if:
    /// 1) Can not create/retrieve prepared statement
    /// 2) Can not execute statement
    pub async fn inner_execute<'a>(
        &'a self,
        querystring: String,
        parameters: Vec<PythonDTO>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client_arc = self.db_client.clone();

        let db_client_guard = db_client_arc.read().await;

        let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(parameters.len());
        for param in parameters.iter() {
            vec_parameters.push(param);
        }

        let statement: tokio_postgres::Statement =
            db_client_guard.prepare_cached(&querystring).await?;

        let result = db_client_guard
            .query(&statement, &vec_parameters.into_boxed_slice())
            .await?;

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    pub fn inner_transaction<'a>(&'a self, isolation_level: Option<IsolationLevel>) -> Transaction {
        let inner_transaction = RustTransaction {
            db_client: self.db_client.clone(),
            is_started: Arc::new(tokio::sync::RwLock::new(false)),
            is_done: Arc::new(tokio::sync::RwLock::new(false)),
            rollback_savepoint: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
            isolation_level: isolation_level,
        };

        Transaction {
            transaction: Arc::new(tokio::sync::RwLock::new(inner_transaction)),
        }
    }
}

#[pyclass]
pub struct Connection(pub Arc<tokio::sync::RwLock<RustConnection>>);

#[pymethods]
impl Connection {
    /// Execute querystring with parameters.
    ///
    /// It converts incoming parameters to rust readable
    /// and then execute the query with them.
    ///
    /// # Errors:
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let connection_arc = self.0.clone();
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let connection_guard = connection_arc.read().await;
            Ok(connection_guard.inner_execute(querystring, params).await?)
        })
    }

    pub fn transaction<'a>(
        &'a self,
        py: Python<'a>,
        isolation_level: Option<IsolationLevel>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let connection_arc = self.0.clone();

        rustengine_future(py, async move {
            let connection_guard = connection_arc.read().await;
            Ok(connection_guard.inner_transaction(isolation_level))
        })
    }
}
