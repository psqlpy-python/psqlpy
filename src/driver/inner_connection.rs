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
    PoolConn(Object),
    SingleConn(Client),
}

impl PsqlpyConnection {
    /// Prepare cached statement.
    ///
    /// # Errors
    /// May return Err if cannot prepare statement.
    pub async fn prepare(&self, query: &str) -> PSQLPyResult<Statement> {
        match self {
            PsqlpyConnection::PoolConn(pconn) => return Ok(pconn.prepare_cached(query).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.prepare(query).await?),
        }
    }

    /// Delete prepared statement.
    ///
    /// # Errors
    /// May return Err if cannot prepare statement.
    pub async fn drop_prepared(&self, stmt: &Statement) -> PSQLPyResult<()> {
        let query = format!("DEALLOCATE PREPARE {}", stmt.name());
        match self {
            PsqlpyConnection::PoolConn(pconn) => return Ok(pconn.batch_execute(&query).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.batch_execute(&query).await?),
        }
    }

    /// Prepare and delete statement.
    ///
    /// # Errors
    /// Can return Err if cannot prepare statement.
    pub async fn prepare_then_drop(&self, query: &str) -> PSQLPyResult<Vec<Type>> {
        let types: Vec<Type>;

        let stmt = self.prepare(query).await?;
        types = stmt.params().to_vec();
        self.drop_prepared(&stmt).await?;

        Ok(types)
    }

    /// Prepare cached statement.
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
            PsqlpyConnection::PoolConn(pconn) => return Ok(pconn.query(statement, params).await?),
            PsqlpyConnection::SingleConn(sconn) => {
                return Ok(sconn.query(statement, params).await?)
            }
        }
    }

    /// Prepare cached statement.
    ///
    /// # Errors
    /// May return Err if cannot execute statement.
    pub async fn batch_execute(&self, query: &str) -> PSQLPyResult<()> {
        match self {
            PsqlpyConnection::PoolConn(pconn) => return Ok(pconn.batch_execute(query).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.batch_execute(query).await?),
        }
    }

    /// Prepare cached statement.
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
            PsqlpyConnection::PoolConn(pconn) => {
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
                &self.prepare(&statement.sql_stmt()).await.map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
                &statement.params(),
            )
            .await
            .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        } else {
            self.query(statement.sql_stmt(), &statement.params())
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

        let result = if prepared {
            self.query(
                &self.prepare(statement.sql_stmt()).await.map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
                &statement.params(),
            )
            .await
            .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        } else {
            self.query(statement.sql_stmt(), &statement.params())
                .await
                .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
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
                let prepared_stmt = &self.prepare(&statement.sql_stmt()).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement in execute_many, operation rolled back {error}",
                    )));
                }
                self.query(
                    &self.prepare(&statement.sql_stmt()).await?,
                    &statement.params(),
                )
                .await
            } else {
                self.query(statement.sql_stmt(), &statement.params()).await
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
                &self.prepare(&statement.sql_stmt()).await.map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
                &statement.params(),
            )
            .await
            .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        } else {
            self.query_one(statement.sql_stmt(), &statement.params())
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
            PsqlpyConnection::PoolConn(pconn) => return Ok(pconn.copy_in(statement).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.copy_in(statement).await?),
        }
    }
}
