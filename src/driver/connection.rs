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

#[pyclass]
pub struct Connection {
    pub db_client: Arc<tokio::sync::RwLock<Object>>,
}

#[pymethods]
impl Connection {
    pub fn execute<'a>(
        &'a self,
        py: Python<'a>,
        querystring: String,
        parameters: Option<&'a PyAny>,
    ) -> RustPSQLDriverPyResult<&PyAny> {
        let db_client_arc = self.db_client.clone();

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?
        }

        rustengine_future(py, async move {
            let mut vec_parameters: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(params.len());
            for param in params.iter() {
                vec_parameters.push(param);
            }
            let db_client_guard = db_client_arc.read().await;
            let statement: tokio_postgres::Statement =
                db_client_guard.prepare_cached(&querystring).await?;

            let result = db_client_guard
                .query(&statement, &vec_parameters.into_boxed_slice())
                .await?;

            Ok(PSQLDriverPyQueryResult::new(result))
        })
    }

    pub fn transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
    ) -> Transaction {
        let inner_transaction = RustTransaction::new(
            self.db_client.clone(),
            Arc::new(tokio::sync::RwLock::new(false)),
            Arc::new(tokio::sync::RwLock::new(false)),
            Arc::new(tokio::sync::RwLock::new(HashSet::new())),
            isolation_level,
            read_variant,
            Default::default(),
        );

        Transaction {
            transaction: Arc::new(tokio::sync::RwLock::new(inner_transaction)),
        }
    }
}
