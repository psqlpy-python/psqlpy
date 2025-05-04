use pyo3::PyObject;
use tokio::sync::RwLockWriteGuard;
use tokio_postgres::Statement;

use crate::{driver::inner_connection::PsqlpyConnection, exceptions::rust_errors::PSQLPyResult};

use super::{
    cache::{StatementCacheInfo, StatementsCache, STMTS_CACHE},
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
        if !self.prepared {
            {
                let stmt_cache_guard = STMTS_CACHE.read().await;
                if let Some(cached) = stmt_cache_guard.get_cache(&self.querystring) {
                    return self.build_with_cached(cached);
                }
            }
        }

        let stmt_cache_guard = STMTS_CACHE.write().await;
        self.build_no_cached(stmt_cache_guard).await
    }

    fn build_with_cached(self, cached: StatementCacheInfo) -> PSQLPyResult<PsqlpyStatement> {
        let raw_parameters = ParametersBuilder::new(&self.parameters, Some(cached.types()));

        let parameters_names = if let Some(converted_qs) = &cached.query.converted_qs {
            Some(converted_qs.params_names().clone())
        } else {
            None
        };

        let prepared_parameters = raw_parameters.prepare(parameters_names)?;

        return Ok(PsqlpyStatement::new(
            cached.query,
            prepared_parameters,
            None,
        ));
    }

    async fn build_no_cached(
        self,
        cache_guard: RwLockWriteGuard<'_, StatementsCache>,
    ) -> PSQLPyResult<PsqlpyStatement> {
        let mut querystring = QueryString::new(&self.querystring);
        querystring.process_qs();

        let prepared_stmt = self.prepare_query(&querystring, self.prepared).await?;
        let parameters_builder =
            ParametersBuilder::new(&self.parameters, Some(prepared_stmt.params().to_vec()));

        let parameters_names = if let Some(converted_qs) = &querystring.converted_qs {
            Some(converted_qs.params_names().clone())
        } else {
            None
        };

        let prepared_parameters = parameters_builder.prepare(parameters_names)?;

        match self.prepared {
            true => {
                return Ok(PsqlpyStatement::new(
                    querystring,
                    prepared_parameters,
                    Some(prepared_stmt),
                ))
            }
            false => {
                self.write_to_cache(cache_guard, &querystring, &prepared_stmt)
                    .await;
                return Ok(PsqlpyStatement::new(querystring, prepared_parameters, None));
            }
        }
    }

    async fn write_to_cache(
        &self,
        mut cache_guard: RwLockWriteGuard<'_, StatementsCache>,
        query: &QueryString,
        inner_stmt: &Statement,
    ) {
        cache_guard.add_cache(query, inner_stmt);
    }

    async fn prepare_query(&self, query: &QueryString, prepared: bool) -> PSQLPyResult<Statement> {
        self.inner_conn.prepare(query.query(), prepared).await
    }
}
