use postgres_types::{ToSql, Type};
use tokio_postgres::Statement;

use crate::exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError};

use super::{
    parameters::{Column, PreparedParameters},
    query::QueryString,
};

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

    #[must_use]
    pub fn raw_query(&self) -> &str {
        self.query.query()
    }

    /// Return tokio-postgres prepared statement.
    ///
    /// # Errors
    /// May return error if there is no prepared stmt from tokio-postgres.
    pub fn statement_query(&self) -> PSQLPyResult<&Statement> {
        match &self.prepared_statement {
            Some(prepared_stmt) => Ok(prepared_stmt),
            None => Err(RustPSQLDriverError::ConnectionExecuteError(
                "No prepared parameters".into(),
            )),
        }
    }

    #[must_use]
    pub fn params(&self) -> Box<[&(dyn ToSql + Sync)]> {
        self.prepared_parameters.params()
    }

    #[must_use]
    pub fn params_typed(&self) -> Box<[(&(dyn ToSql + Sync), Type)]> {
        self.prepared_parameters.params_typed()
    }

    #[must_use]
    pub fn columns(&self) -> &Vec<Column> {
        self.prepared_parameters.columns()
    }

    #[must_use]
    pub fn param_types(&self) -> &[postgres_types::Type] {
        self.prepared_parameters.types()
    }

    /// Return parameter placeholder names extracted from the query string.
    ///
    /// Returns `None` when the query uses positional `$1` syntax, and
    /// `Some(&[String])` for kwargs-style `%(name)s` queries.
    #[must_use]
    pub fn param_names(&self) -> Option<&[String]> {
        self.query
            .converted_qs
            .as_ref()
            .map(|c| c.params_names().as_slice())
    }

    #[must_use]
    pub fn into_prepared_parameters(self) -> PreparedParameters {
        self.prepared_parameters
    }
}
