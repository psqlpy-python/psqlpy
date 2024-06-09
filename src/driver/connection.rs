use deadpool_postgres::Object;
use pyo3::{pyclass, pymethods, Py, PyAny, Python};
use std::{collections::HashSet, sync::Arc, vec};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, postgres_to_py, PythonDTO, QueryParameter},
};

use super::{
    cursor::Cursor,
    transaction::Transaction,
    transaction_options::{IsolationLevel, ReadVariant},
};

#[pyclass]
pub struct Connection {
    db_client: Arc<Object>,
}

impl Connection {
    #[must_use]
    pub fn new(db_client: Object) -> Self {
        Connection {
            db_client: Arc::new(db_client),
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
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query(
                    &db_client
                        .prepare_cached(&querystring)
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot prepare statement, error - {err}"
                            ))
                        })?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
        } else {
            db_client
                .query(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
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
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());
        let mut params: Vec<Vec<PythonDTO>> = vec![];
        if let Some(parameters) = parameters {
            for vec_of_py_any in parameters {
                params.push(convert_parameters(vec_of_py_any)?);
            }
        }
        let prepared = prepared.unwrap_or(true);

        db_client.batch_execute("BEGIN;").await.map_err(|err| {
            RustPSQLDriverError::TransactionBeginError(format!(
                "Cannot start transaction to run execute_many: {err}"
            ))
        })?;
        for param in params {
            let querystring_result = if prepared {
                let prepared_stmt = &db_client.prepare_cached(&querystring).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::TransactionExecuteError(format!(
                        "Cannot prepare statement in execute_many, operation rolled back {error}",
                    )));
                }
                db_client
                    .query(
                        &db_client.prepare_cached(&querystring).await?,
                        &param
                            .iter()
                            .map(|param| param as &QueryParameter)
                            .collect::<Vec<&QueryParameter>>()
                            .into_boxed_slice(),
                    )
                    .await
            } else {
                db_client
                    .query(
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
                db_client.batch_execute("ROLLBACK;").await?;
                return Err(RustPSQLDriverError::TransactionExecuteError(format!(
                    "Error occured in `execute_many` statement, transaction is rolled back: {error}"
                )));
            }
        }

        db_client.batch_execute("COMMIT;").await?;

        Ok(())
    }

    /// Fetch result from the database.
    ///
    /// # Errors
    ///
    /// May return Err Result if
    /// 1) Cannot convert incoming parameters
    /// 2) Cannot prepare statement
    /// 3) Cannot execute query
    pub async fn fetch(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query(
                    &db_client
                        .prepare_cached(&querystring)
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot prepare statement, error - {err}"
                            ))
                        })?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
        } else {
            db_client
                .query(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
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
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query_one(
                    &db_client
                        .prepare_cached(&querystring)
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot prepare statement, error - {err}"
                            ))
                        })?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
        } else {
            db_client
                .query_one(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
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
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            db_client
                .query_one(
                    &db_client
                        .prepare_cached(&querystring)
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot prepare statement, error - {err}"
                            ))
                        })?,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
        } else {
            db_client
                .query_one(
                    &querystring,
                    &params
                        .iter()
                        .map(|param| param as &QueryParameter)
                        .collect::<Vec<&QueryParameter>>()
                        .into_boxed_slice(),
                )
                .await
                .map_err(|err| {
                    RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot execute statement, error - {err}"
                    ))
                })?
        };

        Python::with_gil(|gil| match result.columns().first() {
            Some(first_column) => postgres_to_py(gil, &result, first_column, 0, &None),
            None => Ok(gil.None()),
        })
    }

    #[must_use]
    pub fn transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> Transaction {
        Transaction::new(
            self.db_client.clone(),
            false,
            false,
            isolation_level,
            read_variant,
            deferrable,
            HashSet::new(),
        )
    }

    #[must_use]
    pub fn cursor(
        &self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        fetch_number: Option<usize>,
        scroll: Option<bool>,
        prepared: Option<bool>,
    ) -> Cursor {
        Cursor::new(
            self.db_client.clone(),
            querystring,
            parameters,
            "cur_name".into(),
            fetch_number.unwrap_or(10),
            scroll,
            prepared,
        )
    }
}
