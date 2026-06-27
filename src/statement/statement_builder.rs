use pyo3::PyObject;
use tokio_postgres::Statement;

use crate::{
    connection::{structs::PSQLPyConnection, traits::Connection},
    exceptions::rust_errors::PSQLPyResult,
};

use super::{
    parameters::{Column, ParametersBuilder},
    query::QueryString,
    statement::PsqlpyStatement,
};

pub struct StatementBuilder<'a> {
    querystring: &'a String,
    parameters: &'a Option<PyObject>,
    inner_conn: &'a PSQLPyConnection,
    prepared: bool,
}

impl<'a> StatementBuilder<'a> {
    #[must_use]
    pub fn new(
        querystring: &'a String,
        parameters: &'a Option<PyObject>,
        inner_conn: &'a PSQLPyConnection,
        prepared: Option<bool>,
    ) -> Self {
        Self {
            querystring,
            parameters,
            inner_conn,
            prepared: prepared.unwrap_or(true),
        }
    }

    /// Build new internal statement.
    ///
    /// # Errors
    /// May return error if cannot prepare statement.
    pub async fn build(self) -> PSQLPyResult<PsqlpyStatement> {
        let mut querystring = QueryString::new(self.querystring);
        querystring.process_qs();

        let stmt = self.prepare_query(&querystring, self.prepared).await?;

        let columns = stmt
            .columns()
            .iter()
            .map(|c| Column::new(c.name().to_string(), c.table_oid()))
            .collect::<Vec<Column>>();

        let params_builder = ParametersBuilder::new(
            self.parameters.as_ref(),
            Some(stmt.params().to_vec()),
            columns,
        );

        let param_names = querystring
            .converted_qs
            .as_ref()
            .map(|c| c.params_names().clone());

        let prepared_parameters = params_builder.prepare(param_names)?;

        if self.prepared {
            Ok(PsqlpyStatement::new(
                querystring,
                prepared_parameters,
                Some(stmt),
            ))
        } else {
            Ok(PsqlpyStatement::new(querystring, prepared_parameters, None))
        }
    }

    async fn prepare_query(&self, query: &QueryString, prepared: bool) -> PSQLPyResult<Statement> {
        self.inner_conn.prepare(query.query(), prepared).await
    }
}
