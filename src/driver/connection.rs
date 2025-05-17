use bytes::BytesMut;
use deadpool_postgres::Pool;
use futures_util::pin_mut;
use pyo3::{buffer::PyBuffer, pyclass, pyfunction, pymethods, Py, PyAny, PyErr, Python};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{binary_copy::BinaryCopyInWriter, Config};

use crate::{
    connection::{
        structs::{PSQLPyConnection, PoolConnection},
        traits::Connection as _,
    },
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    format_helpers::quote_ident,
    options::{IsolationLevel, LoadBalanceHosts, ReadVariant, SslMode, TargetSessionAttrs},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    runtime::tokio_runtime,
};

use super::{
    connection_pool::connect_pool, cursor::Cursor, portal::Portal, transaction::Transaction,
};

/// Make new connection pool.
///
/// # Errors
/// May return error if cannot build new connection pool.
#[pyfunction]
#[pyo3(signature = (
    dsn=None,
    username=None,
    password=None,
    host=None,
    hosts=None,
    port=None,
    ports=None,
    db_name=None,
    target_session_attrs=None,
    options=None,
    application_name=None,
    connect_timeout_sec=None,
    connect_timeout_nanosec=None,
    tcp_user_timeout_sec=None,
    tcp_user_timeout_nanosec=None,
    keepalives=None,
    keepalives_idle_sec=None,
    keepalives_idle_nanosec=None,
    keepalives_interval_sec=None,
    keepalives_interval_nanosec=None,
    keepalives_retries=None,
    load_balance_hosts=None,
    ssl_mode=None,
    ca_file=None,
))]
#[allow(clippy::too_many_arguments)]
pub async fn connect(
    dsn: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    hosts: Option<Vec<String>>,
    port: Option<u16>,
    ports: Option<Vec<u16>>,
    db_name: Option<String>,
    target_session_attrs: Option<TargetSessionAttrs>,
    options: Option<String>,
    application_name: Option<String>,
    connect_timeout_sec: Option<u64>,
    connect_timeout_nanosec: Option<u32>,
    tcp_user_timeout_sec: Option<u64>,
    tcp_user_timeout_nanosec: Option<u32>,
    keepalives: Option<bool>,
    keepalives_idle_sec: Option<u64>,
    keepalives_idle_nanosec: Option<u32>,
    keepalives_interval_sec: Option<u64>,
    keepalives_interval_nanosec: Option<u32>,
    keepalives_retries: Option<u32>,
    load_balance_hosts: Option<LoadBalanceHosts>,
    ssl_mode: Option<SslMode>,
    ca_file: Option<String>,
) -> PSQLPyResult<Connection> {
    let mut connection_pool = connect_pool(
        dsn,
        username,
        password,
        host,
        hosts,
        port,
        ports,
        db_name,
        target_session_attrs,
        options,
        application_name,
        connect_timeout_sec,
        connect_timeout_nanosec,
        tcp_user_timeout_sec,
        tcp_user_timeout_nanosec,
        keepalives,
        keepalives_idle_sec,
        keepalives_idle_nanosec,
        keepalives_interval_sec,
        keepalives_interval_nanosec,
        keepalives_retries,
        load_balance_hosts,
        ssl_mode,
        ca_file,
        Some(2),
        None,
    )?;

    let db_connection = tokio_runtime()
        .spawn(async move { connection_pool.retrieve_connection().await })
        .await??;

    Ok(db_connection)
}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct Connection {
    db_client: Option<Arc<RwLock<PSQLPyConnection>>>,
    db_pool: Option<Pool>,
    pub pg_config: Arc<Config>,
}

impl Connection {
    #[must_use]
    pub fn new(
        db_client: Option<Arc<RwLock<PSQLPyConnection>>>,
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
    pub fn db_client(&self) -> Option<Arc<RwLock<PSQLPyConnection>>> {
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
    async fn __aenter__<'a>(self_: Py<Self>) -> PSQLPyResult<Py<Self>> {
        let (db_client, db_pool, pg_config) = pyo3::Python::with_gil(|gil| {
            let self_ = self_.borrow(gil);
            (
                self_.db_client.clone(),
                self_.db_pool.clone(),
                self_.pg_config.clone(),
            )
        });

        if db_client.is_some() {
            return Ok(self_);
        }

        if let Some(db_pool) = db_pool {
            let connection = tokio_runtime()
                .spawn(async move {
                    Ok::<deadpool_postgres::Object, RustPSQLDriverError>(db_pool.get().await?)
                })
                .await??;
            pyo3::Python::with_gil(|gil| {
                let mut self_ = self_.borrow_mut(gil);
                self_.db_client = Some(Arc::new(RwLock::new(PSQLPyConnection::PoolConn(
                    PoolConnection::new(connection, pg_config),
                ))));
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
    ) -> PSQLPyResult<()> {
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
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            let read_conn_g = db_client.read().await;
            let res = read_conn_g.execute(querystring, parameters, prepared).await;
            return res;
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
    pub async fn execute_batch(self_: pyo3::Py<Self>, querystring: String) -> PSQLPyResult<()> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            let read_conn_g = db_client.read().await;
            return read_conn_g.batch_execute(&querystring).await;
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
    ) -> PSQLPyResult<()> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            let read_conn_g = db_client.read().await;
            return read_conn_g
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
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            let read_conn_g = db_client.read().await;
            return read_conn_g.execute(querystring, parameters, prepared).await;
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
    ) -> PSQLPyResult<PSQLDriverSinglePyQueryResult> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            let read_conn_g = db_client.read().await;
            return read_conn_g
                .fetch_row(querystring, parameters, prepared)
                .await;
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
    ) -> PSQLPyResult<Py<PyAny>> {
        let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).db_client.clone());

        if let Some(db_client) = db_client {
            let read_conn_g = db_client.read().await;
            return read_conn_g
                .fetch_val(querystring, parameters, prepared)
                .await;
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
    ))]
    pub fn transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> PSQLPyResult<Transaction> {
        let Some(conn) = &self.db_client else {
            return Err(RustPSQLDriverError::ConnectionClosedError);
        };
        Ok(Transaction::new(
            Some(conn.clone()),
            self.pg_config.clone(),
            isolation_level,
            read_variant,
            deferrable,
        ))
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
    ) -> PSQLPyResult<Cursor> {
        let Some(conn) = &self.db_client else {
            return Err(RustPSQLDriverError::ConnectionClosedError);
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

    #[pyo3(signature = (
        querystring,
        parameters=None,
        fetch_number=None,
    ))]
    pub fn portal(
        &self,
        querystring: String,
        parameters: Option<Py<PyAny>>,
        fetch_number: Option<i32>,
    ) -> PSQLPyResult<Portal> {
        println!("{:?}", fetch_number);
        Ok(Portal::new(
            self.db_client.clone(),
            querystring,
            parameters,
            fetch_number,
        ))
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
    ) -> PSQLPyResult<u64> {
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
