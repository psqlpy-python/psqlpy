use deadpool_postgres::Object;
use postgres_types::ToSql;
use pyo3::{pyclass, pymethods, Py, PyAny, Python};
use std::{collections::HashSet, sync::Arc, vec};
use tokio_postgres::Client;

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, postgres_to_py, PythonDTO, QueryParameter},
};

use super::{
    transaction::Transaction,
    transaction_options::{IsolationLevel, ReadVariant},
};

pub enum ConnectionVar {
    Pool(Object),
    SingleConn(Client),
}

impl ConnectionVar {
    async fn prepare_stmt_cached(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error> {
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.prepare(query).await,
            ConnectionVar::SingleConn(single_conn) => single_conn.prepare(query).await,
        }
    }

    async fn query_qs<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement,
    {
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.query(statement, params).await,
            ConnectionVar::SingleConn(single_conn) => single_conn.query(statement, params).await,
        }
    }

    async fn query_qs_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement,
    {
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.query_one(statement, params).await,
            ConnectionVar::SingleConn(single_conn) => {
                single_conn.query_one(statement, params).await
            }
        }
    }

    async fn batch_execute_qs(&self, query: &str) -> Result<(), tokio_postgres::Error> {
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.batch_execute(query).await,
            ConnectionVar::SingleConn(single_conn) => single_conn.batch_execute(query).await,
        }
    }

    async fn start_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let mut querystring = "START TRANSACTION".to_string();

        if let Some(level) = isolation_level {
            let level = &level.to_str_level();
            querystring.push_str(format!(" ISOLATION LEVEL {level}").as_str());
        };

        querystring.push_str(match read_variant {
            Some(ReadVariant::ReadOnly) => " READ ONLY",
            Some(ReadVariant::ReadWrite) => " READ WRITE",
            None => "",
        });

        querystring.push_str(match deferrable {
            Some(true) => " DEFERRABLE",
            Some(false) => " NOT DEFERRABLE",
            None => "",
        });
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.batch_execute(&querystring).await?,
            ConnectionVar::SingleConn(single_conn) => {
                single_conn.batch_execute(&querystring).await?
            }
        }

        Ok(())
    }

    async fn commit(&self) -> RustPSQLDriverPyResult<()> {
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.batch_execute("COMMIT;").await?,
            ConnectionVar::SingleConn(single_conn) => single_conn.batch_execute("COMMIT;").await?,
        };
        Ok(())
    }
    async fn rollback(&self) -> RustPSQLDriverPyResult<()> {
        match self {
            ConnectionVar::Pool(pool_conn) => pool_conn.batch_execute("ROLLBACK;").await?,
            ConnectionVar::SingleConn(single_conn) => {
                single_conn.batch_execute("ROLLBACK;").await?
            }
        };
        Ok(())
    }
}

#[pyclass]
pub struct Connection {
    connection: Arc<ConnectionVar>,
}

impl Connection {
    #[must_use]
    pub fn new(connection: ConnectionVar) -> Self {
        Connection {
            connection: Arc::new(connection),
        }
    }
}

#[pymethods]
impl Connection {
    /// Execute statement with or witout parameters.
    ///
    /// # Errors
    ///
    /// May return Err Result if
    /// 1) Cannot convert incoming parameters
    /// 2) Cannot prepare statement
    /// 3) Cannot execute query
    pub async fn execute(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).connection.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query_qs(
                    &db_client.prepare_stmt_cached(&querystring).await?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        } else {
            db_client
                .query_qs(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Execute querystring with parameters.
    ///
    /// It converts incoming parameters to rust readable
    /// and then execute the query with them.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    pub async fn execute_many<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).connection.clone());
        let mut params: Vec<Vec<PythonDTO>> = vec![];
        if let Some(parameters) = parameters {
            for vec_of_py_any in parameters {
                params.push(convert_parameters(vec_of_py_any)?);
            }
        }
        let prepared = prepared.unwrap_or(true);

        db_client.batch_execute_qs("BEGIN;").await.map_err(|err| {
            RustPSQLDriverError::DataBaseTransactionError(format!(
                "Cannot start transaction to run execute_many: {err}"
            ))
        })?;
        for param in params {
            let querystring_result = if prepared {
                let prepared_stmt = &db_client.prepare_stmt_cached(&querystring).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::DataBaseTransactionError(format!(
                        "Cannot prepare statement in execute_many, operation rolled back {error}",
                    )));
                }
                db_client
                    .query_qs(
                        &db_client.prepare_stmt_cached(&querystring).await?,
                        &param
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await
            } else {
                db_client
                    .query_qs(
                        &querystring,
                        &param
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await
            };

            if let Err(error) = querystring_result {
                db_client.batch_execute_qs("ROLLBACK;").await?;
                return Err(RustPSQLDriverError::DataBaseTransactionError(format!(
                    "Error occured in `execute_many` statement, transaction is rolled back: {error}"
                )));
            }
        }

        db_client.batch_execute_qs("COMMIT;").await?;

        Ok(())
    }

    /// Fetch exaclty single row from query.
    ///
    /// Method doesn't acquire lock on any structure fields.
    /// It prepares and caches querystring in the inner Object object.
    ///
    /// Then execute the query.
    ///
    /// # Errors
    /// May return Err Result if:
    /// 1) Transaction is not started
    /// 2) Transaction is done already
    /// 3) Can not create/retrieve prepared statement
    /// 4) Can not execute statement
    /// 5) Query returns more than one row
    pub async fn fetch_row(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).connection.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query_qs_one(
                    &db_client.prepare_stmt_cached(&querystring).await?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        } else {
            db_client
                .query_qs_one(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        };

        Ok(PSQLDriverSinglePyQueryResult::new(result))
    }

    /// Execute querystring with parameters and return first value in the first row.
    ///
    /// It converts incoming parameters to rust readable,
    /// executes query with them and returns first row of response.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Cannot convert python parameters
    /// 2) Cannot execute querystring.
    /// 3) Query returns more than one row
    pub async fn fetch_val<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).connection.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query_qs_one(
                    &db_client.prepare_stmt_cached(&querystring).await?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        } else {
            db_client
                .query_qs_one(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await?
        };

        Python::with_gil(|gil| match result.columns().first() {
            Some(first_column) => postgres_to_py(gil, &result, first_column, 0, &None),
            None => Ok(gil.None()),
        })
    }

    // #[must_use]
    // pub fn transaction(
    //     &self,
    //     isolation_level: Option<IsolationLevel>,
    //     read_variant: Option<ReadVariant>,
    //     deferrable: Option<bool>,
    // ) -> Transaction {
    //     Transaction::new(
    //         self.pool_connection.clone(),
    //         false,
    //         false,
    //         isolation_level,
    //         read_variant,
    //         deferrable,
    //         HashSet::new(),
    //     )
    // }
}
