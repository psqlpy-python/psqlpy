use pyo3::PyObject;
use tokio::sync::RwLockWriteGuard;
use tokio_postgres::Statement;

use crate::{
    connection::{structs::PSQLPyConnection, traits::Connection},
    exceptions::rust_errors::PSQLPyResult,
};

use super::{
    cache::{StatementCacheInfo, StatementsCache, STMTS_CACHE},
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
        if !self.prepared {
            {
                let stmt_cache_guard = STMTS_CACHE.read().await;
                if let Some(cached) = stmt_cache_guard.get_cache(self.querystring) {
                    return self.build_with_cached(cached);
                }
            }
        }

        let stmt_cache_guard = STMTS_CACHE.write().await;
        self.build_no_cached(stmt_cache_guard).await
    }

    fn build_with_cached(self, cached: StatementCacheInfo) -> PSQLPyResult<PsqlpyStatement> {
        let raw_parameters = ParametersBuilder::new(
            self.parameters.as_ref(),
            Some(cached.types()),
            cached.columns(),
        );

        let parameters_names = cached
            .query
            .converted_qs
            .as_ref()
            .map(|converted_qs| converted_qs.params_names().clone());

        let prepared_parameters = raw_parameters.prepare(parameters_names)?;

        Ok(PsqlpyStatement::new(
            cached.query,
            prepared_parameters,
            None,
        ))
    }

    async fn build_no_cached(
        self,
        cache_guard: RwLockWriteGuard<'_, StatementsCache>,
    ) -> PSQLPyResult<PsqlpyStatement> {
        let mut querystring = QueryString::new(self.querystring);
        querystring.process_qs();

        let prepared_stmt = self.prepare_query(&querystring, self.prepared).await?;

        let columns = prepared_stmt
            .columns()
            .iter()
            .map(|column| Column::new(column.name().to_string(), column.table_oid()))
            .collect::<Vec<Column>>();
        let parameters_builder = ParametersBuilder::new(
            self.parameters.as_ref(),
            Some(prepared_stmt.params().to_vec()),
            columns,
        );

        let parameters_names = querystring
            .converted_qs
            .as_ref()
            .map(|converted_qs| converted_qs.params_names().clone());

        let prepared_parameters = parameters_builder.prepare(parameters_names)?;

        if self.prepared {
            Ok(PsqlpyStatement::new(
                querystring,
                prepared_parameters,
                Some(prepared_stmt),
            ))
        } else {
            Self::write_to_cache(cache_guard, &querystring, &prepared_stmt);
            Ok(PsqlpyStatement::new(querystring, prepared_parameters, None))
        }
    }

    fn write_to_cache(
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
