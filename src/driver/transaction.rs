use std::sync::Arc;

use bytes::BytesMut;
use futures::{future, pin_mut};
use pyo3::{
    buffer::PyBuffer,
    pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyTuple},
    Py, PyAny, PyErr, PyResult,
};
use tokio::sync::RwLock;
use tokio_postgres::{binary_copy::BinaryCopyInWriter, Config};

use crate::{
    connection::{
        structs::PSQLPyConnection,
        traits::{CloseTransaction, Connection, StartTransaction as _},
    },
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    format_helpers::quote_ident,
    options::{IsolationLevel, ReadVariant},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
};

use super::cursor::Cursor;

#[pyclass(subclass)]
pub struct Transaction {
    pub conn: Option<Arc<RwLock<PSQLPyConnection>>>,
    pub pg_config: Arc<Config>,

    isolation_level: Option<IsolationLevel>,
    read_variant: Option<ReadVariant>,
    deferrable: Option<bool>,
}

impl Transaction {
    pub fn new(
        conn: Option<Arc<RwLock<PSQLPyConnection>>>,
        pg_config: Arc<Config>,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> Self {
        Self {
            conn,
            pg_config,
            isolation_level,
            read_variant,
            deferrable,
        }
    }
}

#[pymethods]
impl Transaction {
    #[must_use]
    pub fn __aiter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __await__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    async fn __aenter__<'a>(self_: Py<Self>) -> PSQLPyResult<Py<Self>> {
        let (isolation_level, read_variant, deferrable, conn) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (
                self_.isolation_level,
                self_.read_variant,
                self_.deferrable,
                self_.conn.clone(),
            )
        });

        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        write_conn_g
            .start_transaction(isolation_level, read_variant, deferrable)
            .await?;

        return Ok(self_);
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __aexit__<'a>(
        self_: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> PSQLPyResult<()> {
        let (conn, is_exception_none, py_err) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (
                self_.conn.clone(),
                exception.is_none(gil),
                PyErr::from_value(exception.into_bound(gil)),
            )
        });

        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        if is_exception_none {
            write_conn_g.commit().await?;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.conn = None;
            });
            Ok(())
        } else {
            write_conn_g.rollback().await?;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.conn = None;
            });
            return Err(RustPSQLDriverError::RustPyError(py_err));
        }
    }

    pub async fn begin(&mut self) -> PSQLPyResult<()> {
        let conn = &self.conn;
        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        write_conn_g
            .start_transaction(self.isolation_level, self.read_variant, self.deferrable)
            .await?;

        Ok(())
    }

    pub async fn commit(&mut self) -> PSQLPyResult<()> {
        let conn = self.conn.clone();
        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        write_conn_g.commit().await?;

        self.conn = None;

        Ok(())
    }

    pub async fn rollback(&mut self) -> PSQLPyResult<()> {
        let conn = self.conn.clone();
        let Some(conn) = conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        let mut write_conn_g = conn.write().await;
        write_conn_g.rollback().await?;

        self.conn = None;

        Ok(())
    }

    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g.execute(querystring, parameters, prepared).await
    }

    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g.execute(querystring, parameters, prepared).await
    }

    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch_val(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .fetch_val(querystring, parameters, prepared)
            .await
    }

    pub async fn execute_batch(&self, querystring: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g.batch_execute(&querystring).await
    }

    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute_many(
        &self,
        querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .execute_many(querystring, parameters, prepared)
            .await
    }

    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch_row(
        &self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverSinglePyQueryResult> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .fetch_row(querystring, parameters, prepared)
            .await
    }

    pub async fn create_savepoint(&mut self, savepoint_name: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .batch_execute(format!("SAVEPOINT {savepoint_name}").as_str())
            .await?;

        Ok(())
    }

    pub async fn release_savepoint(&mut self, savepoint_name: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .batch_execute(format!("RELEASE SAVEPOINT {savepoint_name}").as_str())
            .await?;

        Ok(())
    }

    pub async fn rollback_savepoint(&mut self, savepoint_name: String) -> PSQLPyResult<()> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };

        let read_conn_g = conn.read().await;
        read_conn_g
            .batch_execute(format!("ROLLBACK TO SAVEPOINT {savepoint_name}").as_str())
            .await?;

        Ok(())
    }

    /// Create new cursor object.
    ///
    /// # Errors
    /// May return Err Result if db_client is None
    #[pyo3(signature = (
        querystring,
        parameters=None,
        fetch_number=None,
        scroll=None,
        prepared=None,
    ))]
    pub fn cursor(
        &self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        fetch_number: Option<usize>,
        scroll: Option<bool>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Cursor> {
        let Some(conn) = &self.conn else {
            return Err(RustPSQLDriverError::TransactionClosedError);
        };
        Ok(Cursor::new(
            conn.clone(),
            self.pg_config.clone(),
            querystring,
            parameters,
            fetch_number.unwrap_or(10),
            scroll,
            prepared,
        ))
    }

    #[pyo3(signature = (queries=None, prepared=None))]
    pub async fn pipeline<'py>(
        self_: Py<Self>,
        queries: Option<Py<PyList>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Vec<PSQLDriverPyQueryResult>> {
        let db_client = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);

            self_.conn.clone()
        });

        if let Some(db_client) = db_client {
            let conn_read_g = db_client.read().await;
            let mut futures = vec![];
            if let Some(queries) = queries {
                let gil_result = pyo3::Python::with_gil(|gil| -> PyResult<()> {
                    for single_query in queries.into_bound(gil).iter() {
                        let query_tuple = single_query.downcast::<PyTuple>().map_err(|err| {
                            RustPSQLDriverError::PyToRustValueConversionError(format!(
                                "Cannot cast to tuple: {err}",
                            ))
                        })?;

                        let querystring = query_tuple.get_item(0)?.extract::<String>()?;
                        let params = match query_tuple.get_item(1) {
                            Ok(param) => Some(param.into()),
                            Err(_) => None,
                        };
                        futures.push(conn_read_g.execute(querystring, params, prepared));
                    }
                    Ok(())
                });

                match gil_result {
                    Ok(()) => {}
                    Err(e) => {
                        // Handle PyO3 error, convert to your error type as needed
                        return Err(RustPSQLDriverError::from(e)); // Adjust according to your error types
                    }
                }
            }
            return future::try_join_all(futures).await;
        }

        Err(RustPSQLDriverError::TransactionClosedError)
    }

    /// Perform binary copy to postgres table.
    ///
    /// # Errors
    /// May return Err Result if cannot get bytes,
    /// cannot perform request to the database,
    /// cannot write bytes to the database.
    #[pyo3(signature = (source, table_name, columns=None, schema_name=None))]
    pub async fn binary_copy_to_table(
        self_: pyo3::Py<Self>,
        source: Py<PyAny>,
        table_name: String,
        columns: Option<Vec<String>>,
        schema_name: Option<String>,
    ) -> PSQLPyResult<u64> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).conn.clone());
        let mut table_name = quote_ident(&table_name);
        if let Some(schema_name) = schema_name {
            table_name = format!("{}.{}", quote_ident(&schema_name), table_name);
        }

        let mut formated_columns = String::default();
        if let Some(columns) = columns {
            formated_columns = format!("({})", columns.join(", "));
        }

        let copy_qs = format!("COPY {table_name}{formated_columns} FROM STDIN (FORMAT binary)");

        if let Some(db_client) = db_client {
            let mut psql_bytes: BytesMut = pyo3::Python::with_gil(|gil| {
                let possible_py_buffer: Result<PyBuffer<u8>, PyErr> =
                    source.extract::<PyBuffer<u8>>(gil);
                if let Ok(py_buffer) = possible_py_buffer {
                    let vec_buf = py_buffer.to_vec(gil)?;
                    return Ok(BytesMut::from(vec_buf.as_slice()));
                }

                if let Ok(py_bytes) = source.call_method0(gil, "getvalue") {
                    if let Ok(bytes) = py_bytes.extract::<Vec<u8>>(gil) {
                        return Ok(BytesMut::from(bytes.as_slice()));
                    }
                }

                Err(RustPSQLDriverError::PyToRustValueConversionError(
                    "source must be bytes or support Buffer protocol".into(),
                ))
            })?;

            let read_conn_g = db_client.read().await;
            let sink = read_conn_g.copy_in(&copy_qs).await?;
            let writer = BinaryCopyInWriter::new_empty_buffer(sink, &[]);
            pin_mut!(writer);
            writer.as_mut().write_raw_bytes(&mut psql_bytes).await?;
            let rows_created = writer.as_mut().finish_empty().await?;
            return Ok(rows_created);
        }

        Ok(0)
    }
}
