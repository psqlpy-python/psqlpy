use bytes::BytesMut;
use deadpool_postgres::Pool;
use futures_util::pin_mut;
use pyo3::{buffer::PyBuffer, pyclass, pymethods, Py, PyAny, PyErr, Python};
use std::{collections::HashSet, net::IpAddr, sync::Arc};
use tokio_postgres::{binary_copy::BinaryCopyInWriter, config::Host, Config};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    format_helpers::quote_ident,
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    runtime::tokio_runtime,
};

use super::{
    cursor::Cursor,
    inner_connection::PsqlpyConnection,
    transaction::Transaction,
    transaction_options::{IsolationLevel, ReadVariant, SynchronousCommit},
};

#[pyclass(subclass)]
#[derive(Clone)]
pub struct Connection {
    db_client: Option<Arc<PsqlpyConnection>>,
    db_pool: Option<Pool>,
    pg_config: Arc<Config>,
}

impl Connection {
    #[must_use]
    pub fn new(
        db_client: Option<Arc<PsqlpyConnection>>,
        db_pool: Option<Pool>,
        pg_config: Arc<Config>,
    ) -> Self {
        Connection {
            db_client,
            db_pool,
            pg_config,
        }
    }

    #[must_use]
    pub fn db_client(&self) -> Option<Arc<PsqlpyConnection>> {
        self.db_client.clone()
    }

    #[must_use]
    pub fn db_pool(&self) -> Option<Pool> {
        self.db_pool.clone()
    }
}

impl Default for Connection {
    fn default() -> Self {
        Connection::new(None, None, Arc::new(Config::default()))
    }
}

#[pymethods]
impl Connection {
    #[getter]
    fn conn_dbname(&self) -> Option<&str> {
        self.pg_config.get_dbname()
    }

    #[getter]
    fn user(&self) -> Option<&str> {
        self.pg_config.get_user()
    }

    #[getter]
    fn host_addrs(&self) -> Vec<String> {
        let mut host_addrs_vec = vec![];

        let host_addrs = self.pg_config.get_hostaddrs();
        for ip_addr in host_addrs {
            match ip_addr {
                IpAddr::V4(ipv4) => {
                    host_addrs_vec.push(ipv4.to_string());
                }
                IpAddr::V6(ipv6) => {
                    host_addrs_vec.push(ipv6.to_string());
                }
            }
        }

        host_addrs_vec
    }

    #[cfg(unix)]
    #[getter]
    fn hosts(&self) -> Vec<String> {
        let mut hosts_vec = vec![];

        let hosts = self.pg_config.get_hosts();
        for host in hosts {
            match host {
                Host::Tcp(host) => {
                    hosts_vec.push(host.to_string());
                }
                Host::Unix(host) => {
                    hosts_vec.push(host.display().to_string());
                }
            }
        }

        hosts_vec
    }

    #[cfg(not(unix))]
    #[getter]
    fn hosts(&self) -> Vec<String> {
        let mut hosts_vec = vec![];

        let hosts = self.pg_config.get_hosts();
        for host in hosts {
            match host {
                Host::Tcp(host) => {
                    hosts_vec.push(host.to_string());
                }
                _ => unreachable!(),
            }
        }

        hosts_vec
    }

    #[getter]
    fn ports(&self) -> Vec<&u16> {
        return self.pg_config.get_ports().iter().collect::<Vec<&u16>>();
    }

    #[getter]
    fn options(&self) -> Option<&str> {
        return self.pg_config.get_options();
    }

    async fn __aenter__<'a>(self_: Py<Self>) -> RustPSQLDriverPyResult<Py<Self>> {
        let (db_client, db_pool) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (self_.db_client.clone(), self_.db_pool.clone())
        });

        if db_client.is_some() {
            return Ok(self_);
        }

        if let Some(db_pool) = db_pool {
            let db_connection = tokio_runtime()
                .spawn(async move {
                    Ok::<deadpool_postgres::Object, RustPSQLDriverError>(db_pool.get().await?)
                })
                .await??;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.db_client = Some(Arc::new(PsqlpyConnection::PoolConn(db_connection)));
            });
            return Ok(self_);
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
    }

    #[allow(clippy::unused_async)]
    async fn __aexit__<'a>(
        self_: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<()> {
        let (is_exception_none, py_err) = pyo3::Python::with_gil(|gil| {
            (
                exception.is_none(gil),
                PyErr::from_value(exception.into_bound(gil)),
            )
        });

        pyo3::Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);

            std::mem::take(&mut self_.db_client);
            std::mem::take(&mut self_.db_pool);

            if is_exception_none {
                Ok(())
            } else {
                Err(RustPSQLDriverError::RustPyError(py_err))
            }
        })
    }

    /// Execute statement with or witout parameters.
    ///
    /// # Errors
    ///
    /// May return Err Result if
    /// 1) Cannot convert incoming parameters
    /// 2) Cannot prepare statement
    /// 3) Cannot execute query
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            return db_client.execute(querystring, parameters, prepared).await;
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
    }

    /// Executes a sequence of SQL statements using the simple query protocol.
    ///
    /// Statements should be separated by semicolons.
    /// If an error occurs, execution of the sequence will stop at that point.
    /// This is intended for use when, for example,
    /// initializing a database schema.
    ///
    /// # Errors
    ///
    /// May return Err Result if:
    /// 1) Connection is closed.
    /// 2) Cannot execute querystring.
    pub async fn execute_batch(
        self_: pyo3::Py<Self>,
        querystring: String,
    ) -> RustPSQLDriverPyResult<()> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            return db_client.batch_execute(&querystring).await;
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
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
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute_many<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<Vec<Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            return db_client
                .execute_many(querystring, parameters, prepared)
                .await;
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
    }

    /// Fetch result from the database.
    ///
    /// # Errors
    ///
    /// May return Err Result if
    /// 1) Cannot convert incoming parameters
    /// 2) Cannot prepare statement
    /// 3) Cannot execute query
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            return db_client.execute(querystring, parameters, prepared).await;
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
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
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch_row(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            return db_client.fetch_row(querystring, parameters, prepared).await;
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
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
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch_val<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            return db_client.fetch_val(querystring, parameters, prepared).await;
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
    }

    /// Create new transaction object.
    ///
    /// # Errors
    /// May return Err Result if db_client is None.
    #[pyo3(signature = (
        isolation_level=None,
        read_variant=None,
        deferrable=None,
        synchronous_commit=None,
    ))]
    pub fn transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
        synchronous_commit: Option<SynchronousCommit>,
    ) -> RustPSQLDriverPyResult<Transaction> {
        if let Some(db_client) = &self.db_client {
            return Ok(Transaction::new(
                db_client.clone(),
                self.pg_config.clone(),
                false,
                false,
                isolation_level,
                synchronous_commit,
                read_variant,
                deferrable,
                HashSet::new(),
            ));
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
    }

    /// Create new cursor object.
    ///
    /// # Errors
    /// May return Err Result if db_client is None.
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
    ) -> RustPSQLDriverPyResult<Cursor> {
        if let Some(db_client) = &self.db_client {
            return Ok(Cursor::new(
                db_client.clone(),
                self.pg_config.clone(),
                querystring,
                parameters,
                "cur_name".into(),
                fetch_number.unwrap_or(10),
                scroll,
                prepared,
            ));
        }

        Err(RustPSQLDriverError::ConnectionClosedError)
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn back_to_pool(self_: pyo3::Py<Self>) {
        pyo3::Python::with_gil(|gil| {
            let mut connection = self_.borrow_mut(gil);
            if connection.db_client.is_some() {
                std::mem::take(&mut connection.db_client);
            }
        });
    }

    /// Perform binary copy to postgres table.
    ///
    /// # Errors
    /// May return Err Result if cannot get bytes,
    /// cannot perform request to the database,
    /// cannot write bytes to the database.
    #[pyo3(signature = (
        source,
        table_name,
        columns=None,
        schema_name=None,
    ))]
    pub async fn binary_copy_to_table(
        self_: pyo3::Py<Self>,
        source: Py<PyAny>,
        table_name: String,
        columns: Option<Vec<String>>,
        schema_name: Option<String>,
    ) -> RustPSQLDriverPyResult<u64> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());
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
            let mut psql_bytes: BytesMut = Python::with_gil(|gil| {
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

            let sink = db_client.copy_in(&copy_qs).await?;
            let writer = BinaryCopyInWriter::new_empty_buffer(sink, &[]);
            pin_mut!(writer);
            writer.as_mut().write_raw_bytes(&mut psql_bytes).await?;
            let rows_created = writer.as_mut().finish_empty().await?;
            return Ok(rows_created);
        }

        Ok(0)
    }
}
