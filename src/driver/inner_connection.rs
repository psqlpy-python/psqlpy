use bytes::Buf;
use deadpool_postgres::Object;
use postgres_types::ToSql;
use pyo3::{Py, PyAny, Python};
use std::vec;
use tokio_postgres::{Client, CopyInSink, Row, Statement, ToStatement};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters_and_qs, postgres_to_py, PythonDTO, QueryParameter},
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
    pub async fn prepare_cached(&self, query: &str) -> RustPSQLDriverPyResult<Statement> {
        match self {
            PsqlpyConnection::PoolConn(pconn) => return Ok(pconn.prepare_cached(query).await?),
            PsqlpyConnection::SingleConn(sconn) => return Ok(sconn.prepare(query).await?),
        }
    }

    /// Prepare cached statement.
    ///
    /// # Errors
    /// May return Err if cannot execute statement.
    pub async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> RustPSQLDriverPyResult<Vec<Row>>
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
    pub async fn batch_execute(&self, query: &str) -> RustPSQLDriverPyResult<()> {
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
    ) -> RustPSQLDriverPyResult<Row>
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
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let prepared = prepared.unwrap_or(true);

        let (qs, params) = convert_parameters_and_qs(querystring, parameters)?;

        let boxed_params = &params
            .iter()
            .map(|param| param as &QueryParameter)
            .collect::<Vec<&QueryParameter>>()
            .into_boxed_slice();

        let result = if prepared {
            self.query(
                &self.prepare_cached(&qs).await.map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
                boxed_params,
            )
            .await
            .map_err(|err| {
                RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Cannot execute statement, error - {err}"
                ))
            })?
        } else {
            self.query(&qs, boxed_params).await.map_err(|err| {
                RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Cannot execute statement, error - {err}"
                ))
            })?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    pub async fn execute(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let prepared = prepared.unwrap_or(true);

        let (qs, params) = convert_parameters_and_qs(querystring, parameters)?;

        let boxed_params = &params
            .iter()
            .map(|param| param as &QueryParameter)
            .collect::<Vec<&QueryParameter>>()
            .into_boxed_slice();

        let result = if prepared {
            self.query(
                &self.prepare_cached(&qs).await.map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
                boxed_params,
            )
            .await
            .map_err(|err| {
                RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Cannot execute statement, error - {err}"
                ))
            })?
        } else {
            self.query(&qs, boxed_params).await.map_err(|err| {
                RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Cannot execute statement, error - {err}"
                ))
            })?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    pub async fn execute_many(
        &self,
        mut querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let prepared = prepared.unwrap_or(true);

        let mut params: Vec<Vec<PythonDTO>> = vec![];
        if let Some(parameters) = parameters {
            for vec_of_py_any in parameters {
                // TODO: Fix multiple qs creation
                let (qs, parsed_params) =
                    convert_parameters_and_qs(querystring.clone(), Some(vec_of_py_any))?;
                querystring = qs;
                params.push(parsed_params);
            }
        }

        for param in params {
            let boxed_params = &param
                .iter()
                .map(|param| param as &QueryParameter)
                .collect::<Vec<&QueryParameter>>()
                .into_boxed_slice();

            let querystring_result = if prepared {
                let prepared_stmt = &self.prepare_cached(&querystring).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement in execute_many, operation rolled back {error}",
                    )));
                }
                self.query(&self.prepare_cached(&querystring).await?, boxed_params)
                    .await
            } else {
                self.query(&querystring, boxed_params).await
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
    ) -> RustPSQLDriverPyResult<Row> {
        let prepared = prepared.unwrap_or(true);

        let (qs, params) = convert_parameters_and_qs(querystring, parameters)?;

        let boxed_params = &params
            .iter()
            .map(|param| param as &QueryParameter)
            .collect::<Vec<&QueryParameter>>()
            .into_boxed_slice();

        let result = if prepared {
            self.query_one(
                &self.prepare_cached(&qs).await.map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement, error - {err}"
                    ))
                })?,
                boxed_params,
            )
            .await
            .map_err(|err| {
                RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Cannot execute statement, error - {err}"
                ))
            })?
        } else {
            self.query_one(&qs, boxed_params).await.map_err(|err| {
                RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Cannot execute statement, error - {err}"
                ))
            })?
        };

        return Ok(result);
    }

    pub async fn fetch_row(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
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
    ) -> RustPSQLDriverPyResult<Py<PyAny>> {
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
    pub async fn copy_in<T, U>(&self, statement: &T) -> RustPSQLDriverPyResult<CopyInSink<U>>
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
