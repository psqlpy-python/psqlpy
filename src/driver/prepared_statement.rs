use std::sync::Arc;

use pyo3::{pyclass, pymethods};
use tokio::sync::RwLock;
use tokio_postgres::Config;

use crate::{
    connection::structs::PSQLPyConnection,
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    query_result::PSQLDriverPyQueryResult,
    statement::{parameters::Column, statement::PsqlpyStatement},
};

use super::cursor::Cursor;

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PreparedStatement {
    pub conn: Option<Arc<RwLock<PSQLPyConnection>>>,
    pub pg_config: Arc<Config>,
    statement: PsqlpyStatement,
}

impl PreparedStatement {
    pub fn new(
        conn: Option<Arc<RwLock<PSQLPyConnection>>>,
        pg_config: Arc<Config>,
        statement: PsqlpyStatement,
    ) -> Self {
        Self {
            conn,
            pg_config,
            statement,
        }
    }
}

#[pymethods]
impl PreparedStatement {
    async fn execute(&self) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError("12".into()));
        };

        let read_conn_g = conn.read().await;
        read_conn_g.execute_statement(&self.statement).await
    }

    fn cursor(&self) -> PSQLPyResult<Cursor> {
        Ok(Cursor::new(
            self.conn.clone(),
            None,
            None,
            None,
            self.pg_config.clone(),
            Some(self.statement.clone()),
        ))
    }

    fn columns(&self) -> Vec<Column> {
        self.statement.columns().clone()
    }
}
