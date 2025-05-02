use pyo3::PyObject;
use tokio_postgres::Statement;

use crate::{driver::inner_connection::PsqlpyConnection, exceptions::rust_errors::PSQLPyResult};

use super::{
    cache::{StatementCacheInfo, STMTS_CACHE},
    parameters::ParametersBuilder,
    query::QueryString,
    statement::PsqlpyStatement,
};

pub struct StatementBuilder<'a> {
    querystring: String,
    parameters: Option<PyObject>,
    inner_conn: &'a PsqlpyConnection,
    prepared: bool,
}

impl<'a> StatementBuilder<'a> {
    pub fn new(
        querystring: String,
        parameters: Option<PyObject>,
        inner_conn: &'a PsqlpyConnection,
        prepared: Option<bool>,
    ) -> Self {
        Self {
            querystring,
            parameters,
            inner_conn,
            prepared: prepared.unwrap_or(true),
        }
    }

    pub async fn build(self) -> PSQLPyResult<PsqlpyStatement> {
        {
            let stmt_cache_guard = STMTS_CACHE.read().await;
            if let Some(cached) = stmt_cache_guard.get_cache(&self.querystring) {
                return self.build_with_cached(cached);
            }
        }

        self.build_no_cached().await
    }

    fn build_with_cached(self, cached: StatementCacheInfo) -> PSQLPyResult<PsqlpyStatement> {
        let raw_parameters = ParametersBuilder::new(&self.parameters, Some(cached.types()));

        let parameters_names = if let Some(converted_qs) = &cached.query.converted_qs {
            Some(converted_qs.params_names().clone())
        } else {
            None
        };

        let prepared_parameters = raw_parameters.prepare(parameters_names)?;

        return Ok(PsqlpyStatement::new(cached.query, prepared_parameters));
    }

    async fn build_no_cached(self) -> PSQLPyResult<PsqlpyStatement> {
        let mut querystring = QueryString::new(&self.querystring);
        querystring.process_qs();

        let prepared_stmt = self.prepare_query(&querystring).await?;
        let parameters_builder =
            ParametersBuilder::new(&self.parameters, Some(prepared_stmt.params().to_vec()));

        if !self.prepared {
            Self::drop_prepared(self.inner_conn, &prepared_stmt).await?;
        }

        let parameters_names = if let Some(converted_qs) = &querystring.converted_qs {
            Some(converted_qs.params_names().clone())
        } else {
            None
        };

        let prepared_parameters = parameters_builder.prepare(parameters_names)?;

        {
            self.write_to_cache(&querystring, &prepared_stmt).await;
        }
        let statement = PsqlpyStatement::new(querystring, prepared_parameters);

        return Ok(statement);
    }

    async fn write_to_cache(&self, query: &QueryString, inner_stmt: &Statement) {
        let mut stmt_cache_guard = STMTS_CACHE.write().await;
        stmt_cache_guard.add_cache(query, inner_stmt);
    }

    async fn prepare_query(&self, query: &QueryString) -> PSQLPyResult<Statement> {
        self.inner_conn.prepare(query.query()).await
    }

    async fn drop_prepared(inner_conn: &PsqlpyConnection, stmt: &Statement) -> PSQLPyResult<()> {
        inner_conn.drop_prepared(stmt).await
    }
}
