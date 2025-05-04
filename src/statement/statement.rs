use postgres_types::{ToSql, Type};
use tokio_postgres::Statement;

use crate::exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError};

use super::{parameters::PreparedParameters, query::QueryString};

#[derive(Clone, Debug)]
pub struct PsqlpyStatement {
    query: QueryString,
    prepared_parameters: PreparedParameters,
    prepared_statement: Option<Statement>,
}

impl PsqlpyStatement {
    pub(crate) fn new(
        query: QueryString,
        prepared_parameters: PreparedParameters,
        prepared_statement: Option<Statement>,
    ) -> Self {
        Self {
            query,
            prepared_parameters,
            prepared_statement,
        }
    }

    pub fn raw_query(&self) -> &str {
        self.query.query()
    }

    pub fn statement_query(&self) -> PSQLPyResult<&Statement> {
        match &self.prepared_statement {
            Some(prepared_stmt) => return Ok(prepared_stmt),
            None => return Err(RustPSQLDriverError::ConnectionExecuteError("No".into())),
        }
    }

    pub fn params(&self) -> Box<[&(dyn ToSql + Sync)]> {
        self.prepared_parameters.params()
    }

    pub fn params_typed(&self) -> Box<[(&(dyn ToSql + Sync), Type)]> {
        self.prepared_parameters.params_typed()
    }
}
