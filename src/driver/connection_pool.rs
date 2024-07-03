use crate::runtime::tokio_runtime;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use pyo3::{pyclass, pyfunction, pymethods, PyAny};
use std::{sync::Arc, vec};
use tokio_postgres::NoTls;

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO, QueryParameter},
};

use super::{
    common_options::{ConnRecyclingMethod, LoadBalanceHosts, SslMode, TargetSessionAttrs},
    connection::Connection,
    utils::build_connection_config,
};

/// Make new connection pool.
///
/// # Errors
/// May return error if cannot build new connection pool.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn connect(
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

    max_db_pool_size: Option<usize>,
    conn_recycling_method: Option<ConnRecyclingMethod>,
) -> RustPSQLDriverPyResult<ConnectionPool> {
    if let Some(max_db_pool_size) = max_db_pool_size {
        if max_db_pool_size < 2 {
            return Err(RustPSQLDriverError::ConnectionPoolConfigurationError(
                "Maximum database pool size must be more than 1".into(),
            ));
        }
    }

    let pg_config = build_connection_config(
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
    )?;

    let mgr_config: ManagerConfig;
    if let Some(conn_recycling_method) = conn_recycling_method {
        mgr_config = ManagerConfig {
            recycling_method: conn_recycling_method.to_internal(),
        }
    } else {
        mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
    }

    let mgr: Manager;
    if let Some(ca_file) = ca_file {
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_ca_file(ca_file)?;
        let tls_connector = MakeTlsConnector::new(builder.build());
        mgr = Manager::from_config(pg_config, tls_connector, mgr_config);
    } else {
        mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    }

    let mut db_pool_builder = Pool::builder(mgr);
    if let Some(max_db_pool_size) = max_db_pool_size {
        db_pool_builder = db_pool_builder.max_size(max_db_pool_size);
    }

    let db_pool = db_pool_builder.build()?;

    Ok(ConnectionPool(db_pool))
}

#[pyclass]
#[allow(clippy::module_name_repetitions)]
pub struct ConnectionPoolStatus {
    /// The maximum size of the pool.
    pub max_size: usize,

    /// The current size of the pool.
    pub size: usize,

    /// The number of available objects in the pool.
    pub available: usize,

    /// The number of futures waiting for an object.
    pub waiting: usize,
}

impl ConnectionPoolStatus {
    fn new(max_size: usize, size: usize, available: usize, waiting: usize) -> Self {
        ConnectionPoolStatus {
            max_size,
            size,
            available,
            waiting,
        }
    }
}

#[pymethods]
impl ConnectionPoolStatus {
    #[getter]
    fn get_max_size(&self) -> usize {
        self.max_size
    }

    #[getter]
    fn get_size(&self) -> usize {
        self.size
    }

    #[getter]
    fn get_available(&self) -> usize {
        self.available
    }

    #[getter]
    fn get_waiting(&self) -> usize {
        self.waiting
    }

    fn __str__(&self) -> String {
        format!(
            "Connection Pool Status - [max_size: {}, size: {}, available: {}, waiting: {}]",
            self.max_size, self.size, self.available, self.waiting,
        )
    }
}

#[pyclass]
pub struct ConnectionPool(pub Pool);

#[pymethods]
impl ConnectionPool {
    /// Create new connection pool.
    ///
    /// # Errors
    /// May return error if cannot build new connection pool.
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        max_db_pool_size: Option<usize>,
        conn_recycling_method: Option<ConnRecyclingMethod>,
        ssl_mode: Option<SslMode>,
        ca_file: Option<String>,
    ) -> RustPSQLDriverPyResult<Self> {
        connect(
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
            max_db_pool_size,
            conn_recycling_method,
        )
    }

    #[must_use]
    pub fn status(&self) -> ConnectionPoolStatus {
        let inner_status = self.0.status();

        ConnectionPoolStatus::new(
            inner_status.max_size,
            inner_status.size,
            inner_status.available,
            inner_status.waiting,
        )
    }

    pub fn resize(&self, new_max_size: usize) {
        self.0.resize(new_max_size);
    }

    /// Execute querystring with parameters.
    ///
    /// Prepare statement and cache it, then execute.
    ///
    /// # Errors
    /// May return Err Result if cannot retrieve new connection
    /// or prepare statement or execute statement.
    pub async fn execute<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_pool = pyo3::Python::with_gil(|gil| self_.borrow(gil).0.clone());

        let db_pool_manager = tokio_runtime()
            .spawn(async move { Ok::<Object, RustPSQLDriverError>(db_pool.get().await?) })
            .await??;
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);
        let result = if prepared {
            tokio_runtime()
                .spawn(async move {
                    db_pool_manager
                        .query(
                            &db_pool_manager.prepare_cached(&querystring).await?,
                            &params
                                .iter()
                                .map(|param| param as &QueryParameter)
                                .collect::<Vec<&QueryParameter>>(),
                        )
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot execute statement from ConnectionPool, error - {err}"
                            ))
                        })
                })
                .await??
        } else {
            tokio_runtime()
                .spawn(async move {
                    db_pool_manager
                        .query(
                            &querystring,
                            &params
                                .iter()
                                .map(|param| param as &QueryParameter)
                                .collect::<Vec<&QueryParameter>>(),
                        )
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot execute statement from ConnectionPool, error - {err}"
                            ))
                        })
                })
                .await??
        };
        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Fetch result from the database.
    ///
    /// It's the same as `execute`, we made it for people who prefer
    /// `fetch()`.
    ///
    /// Prepare statement and cache it, then execute.
    ///
    /// # Errors
    /// May return Err Result if cannot retrieve new connection
    /// or prepare statement or execute statement.
    pub async fn fetch<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_pool = pyo3::Python::with_gil(|gil| self_.borrow(gil).0.clone());

        let db_pool_manager = tokio_runtime()
            .spawn(async move { Ok::<Object, RustPSQLDriverError>(db_pool.get().await?) })
            .await??;
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);
        let result = if prepared {
            tokio_runtime()
                .spawn(async move {
                    db_pool_manager
                        .query(
                            &db_pool_manager.prepare_cached(&querystring).await?,
                            &params
                                .iter()
                                .map(|param| param as &QueryParameter)
                                .collect::<Vec<&QueryParameter>>(),
                        )
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot execute statement from ConnectionPool, error - {err}"
                            ))
                        })
                })
                .await??
        } else {
            tokio_runtime()
                .spawn(async move {
                    db_pool_manager
                        .query(
                            &querystring,
                            &params
                                .iter()
                                .map(|param| param as &QueryParameter)
                                .collect::<Vec<&QueryParameter>>(),
                        )
                        .await
                        .map_err(|err| {
                            RustPSQLDriverError::ConnectionExecuteError(format!(
                                "Cannot execute statement from ConnectionPool, error - {err}"
                            ))
                        })
                })
                .await??
        };
        Ok(PSQLDriverPyQueryResult::new(result))
    }

    #[must_use]
    pub fn acquire(&self) -> Connection {
        Connection::new(None, Some(self.0.clone()))
    }

    /// Return new single connection.
    ///
    /// # Errors
    /// May return Err Result if cannot get new connection from the pool.
    pub async fn connection(self_: pyo3::Py<Self>) -> RustPSQLDriverPyResult<Connection> {
        let db_pool = pyo3::Python::with_gil(|gil| self_.borrow(gil).0.clone());
        let db_connection = tokio_runtime()
            .spawn(async move {
                Ok::<deadpool_postgres::Object, RustPSQLDriverError>(db_pool.get().await?)
            })
            .await??;

        Ok(Connection::new(Some(Arc::new(db_connection)), None))
    }

    /// Return new single connection.
    ///
    /// # Errors
    /// May return Err Result if cannot get new connection from the pool.
    pub fn close(&self) {
        let db_pool = self.0.clone();

        db_pool.close();
    }
}
