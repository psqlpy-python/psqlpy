use bytes::Buf;
use deadpool_postgres::Object;
use postgres_types::{ToSql, Type};
use pyo3::{Py, PyAny, Python};
use std::vec;
use tokio_postgres::{Client, CopyInSink, Row, Statement, ToStatement};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    statement::{statement::PsqlpyStatement, statement_builder::StatementBuilder},
    value_converter::to_python::postgres_to_py,
};

#[allow(clippy::module_name_repetitions)]
pub enum PsqlpyConnection {
    PoolConn(Object, bool),
    SingleConn(Client),
}

impl PsqlpyConnection {
    /// Prepare cached statement.
    ///
    /// # Errors
    /// May return Err if cannot prepare statement.
    pub async fn prepare(&self, query: &str, prepared: bool) -> PSQLPyResult<Statement> {
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => {
                if prepared {
                    return Ok(pconn.prepare_cached(query).await?);
                } else {
                    let prepared = pconn.prepare(query).await?;
                    self.drop_prepared(&prepared).await?;
                    return Ok(prepared);
                }
            }
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.prepare(query).await?),
        }
    }

    /// Delete prepared statement.
    ///
    /// # Errors
    /// May return Err if cannot prepare statement.
    pub async fn drop_prepared(&self, stmt: &Statement) -> PSQLPyResult<()> {
        let deallocate_query = format!("DEALLOCATE PREPARE {}", stmt.name());
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => {
                let res = Ok(pconn.batch_execute(&deallocate_query).await?);
                res
            }
            PsqlpyConnection::SingleConn(sconn) => {
                return Ok(sconn.batch_execute(&deallocate_query).await?)
            }
        }
    }

    /// Execute statement with parameters.
    ///
    /// # Errors
    /// May return Err if cannot execute statement.
    pub async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> PSQLPyResult<Vec<Row>>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => {
                return Ok(pconn.query(statement, params).await?)
            }
            PsqlpyConnection::SingleConn(sconn) => {
                return Ok(sconn.query(statement, params).await?)
            }
        }
    }

    /// Execute statement with parameters.
    ///
    /// # Errors
    /// May return Err if cannot execute statement.
    pub async fn query_typed(
        &self,
        statement: &str,
        params: &[(&(dyn ToSql + Sync), Type)],
    ) -> PSQLPyResult<Vec<Row>> {
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => {
                return Ok(pconn.query_typed(statement, params).await?)
            }
            PsqlpyConnection::SingleConn(sconn) => {
                return Ok(sconn.query_typed(statement, params).await?)
            }
        }
    }

    /// Batch execute statement.
    ///
    /// # Errors
    /// May return Err if cannot execute statement.
    pub async fn batch_execute(&self, query: &str) -> PSQLPyResult<()> {
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => return Ok(pconn.batch_execute(query).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.batch_execute(query).await?),
        }
    }

    /// Prepare cached statement.
    ///
    /// # Errors
    /// May return Err if cannot execute copy data.
    pub async fn copy_in<T, U>(&self, statement: &T) -> PSQLPyResult<CopyInSink<U>>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => return Ok(pconn.copy_in(statement).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.copy_in(statement).await?),
        }
    }

    /// Executes a statement which returns a single row, returning it.
    ///
    /// # Errors
    /// May return Err if cannot execute statement.
    pub async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> PSQLPyResult<Row>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            PsqlpyConnection::PoolConn(pconn, _) => {
                return Ok(pconn.query_one(statement, params).await?)
            }
            PsqlpyConnection::SingleConn(sconn) => {
                return Ok(sconn.query_one(statement, params).await?)
            }
        }
    }

    pub async fn cursor_execute(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let statement = StatementBuilder::new(querystring, parameters, self, prepared)
            .build()
            .await?;

        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            self.query(
                &self
                    .prepare(&statement.raw_query(), true)
                    .await
                    .map_err(|err| {
                        RustPSQLDriverError::ConnectionExecuteError(format!(
                            "Cannot prepare statement, error - {err}"
                        ))
                    })?,
                &statement.params(),
            )
            .await
            .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        } else {
            self.query(statement.raw_query(), &statement.params())
                .await
                .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    pub async fn execute(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let statement = StatementBuilder::new(querystring, parameters, self, prepared)
            .build()
            .await?;

        let prepared = prepared.unwrap_or(true);

        let result = match prepared {
            true => self
                .query(statement.statement_query()?, &statement.params())
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
            false => self
                .query_typed(statement.raw_query(), &statement.params_typed())
                .await
                .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?,
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    pub async fn execute_many(
        &self,
        querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<()> {
        let mut statements: Vec<PsqlpyStatement> = vec![];
        if let Some(parameters) = parameters {
            for vec_of_py_any in parameters {
                // TODO: Fix multiple qs creation
                let statement =
                    StatementBuilder::new(querystring.clone(), Some(vec_of_py_any), self, prepared)
                        .build()
                        .await?;

                statements.push(statement);
            }
        }

        let prepared = prepared.unwrap_or(true);

        for statement in statements {
            let querystring_result = if prepared {
                let prepared_stmt = &self.prepare(&statement.raw_query(), true).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement in execute_many, operation rolled back {error}",
                    )));
                }
                self.query(
                    &self.prepare(&statement.raw_query(), true).await?,
                    &statement.params(),
                )
                .await
            } else {
                self.query(statement.raw_query(), &statement.params()).await
            };

            if let Err(error) = querystring_result {
                return Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Error occured in `execute_many` statement: {error}"
                )));
            }
        }

        return Ok(());
    }

    pub async fn fetch_row_raw(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Row> {
        let statement = StatementBuilder::new(querystring, parameters, self, prepared)
            .build()
            .await?;

        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            self.query_one(
                &self
                    .prepare(&statement.raw_query(), true)
                    .await
                    .map_err(|err| {
                        RustPSQLDriverError::ConnectionExecuteError(format!(
                            "Cannot prepare statement, error - {err}"
                        ))
                    })?,
                &statement.params(),
            )
            .await
            .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        } else {
            self.query_one(statement.raw_query(), &statement.params())
                .await
                .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        };

        return Ok(result);
    }

    pub async fn fetch_row(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverSinglePyQueryResult> {
        let result = self
            .fetch_row_raw(querystring, parameters, prepared)
            .await?;

        return Ok(PSQLDriverSinglePyQueryResult::new(result));
    }

    pub async fn fetch_val(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let result = self
            .fetch_row_raw(querystring, parameters, prepared)
            .await?;

        return Python::with_gil(|gil| match result.columns().first() {
            Some(first_column) => postgres_to_py(gil, &result, first_column, 0, &None),
            None => Ok(gil.None()),
        });
    }
}
