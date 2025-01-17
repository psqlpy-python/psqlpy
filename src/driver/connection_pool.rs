use crate::runtime::tokio_runtime;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use pyo3::{pyclass, pyfunction, pymethods, Py, PyAny, Python};
use std::{sync::Arc, time::Duration, vec};
use tokio::time::sleep;
use tokio_postgres::{Config, NoTls};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO, QueryParameter},
};

use super::{
    common_options::{ConnRecyclingMethod, LoadBalanceHosts, SslMode, TargetSessionAttrs},
    connection::Connection,
    listener::Listener,
    utils::{build_connection_config, build_manager, build_tls, ConfiguredTLS},
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
    max_db_pool_size=None,
    conn_recycling_method=None,
))]
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

    let mgr: Manager = build_manager(
        mgr_config,
        pg_config.clone(),
        build_tls(&ca_file, ssl_mode)?,
    );

    let mut db_pool_builder = Pool::builder(mgr);
    if let Some(max_db_pool_size) = max_db_pool_size {
        db_pool_builder = db_pool_builder.max_size(max_db_pool_size);
    }

    let pool = db_pool_builder.build()?;

    Ok(ConnectionPool {
        pool,
        pg_config,
        ca_file,
        ssl_mode,
    })
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

// #[pyclass(subclass)]
// pub struct ConnectionPool(pub Pool);
#[pyclass(subclass)]
pub struct ConnectionPool {
    pool: Pool,
    pg_config: Config,
    ca_file: Option<String>,
    ssl_mode: Option<SslMode>,
}

impl ConnectionPool {
    pub fn build(
        pool: Pool,
        pg_config: Config,
        ca_file: Option<String>,
        ssl_mode: Option<SslMode>,
    ) -> Self {
        ConnectionPool {
            pool,
            pg_config,
            ca_file,
            ssl_mode,
        }
    }
}

#[pymethods]
impl ConnectionPool {
    /// Create new connection pool.
    ///
    /// # Errors
    /// May return error if cannot build new connection pool.
    #[new]
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
        max_db_pool_size=None,
        conn_recycling_method=None,
        ssl_mode=None,
        ca_file=None,
    ))]
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
    pub fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    #[allow(clippy::needless_pass_by_value)]
    fn __enter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    #[allow(clippy::needless_pass_by_value)]
    fn __exit__(
        self_: Py<Self>,
        _exception_type: Py<PyAny>,
        _exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) {
        pyo3::Python::with_gil(|gil| {
            self_.borrow(gil).close();
        });
    }

    #[must_use]
    pub fn status(&self) -> ConnectionPoolStatus {
        let inner_status = self.pool.status();

        ConnectionPoolStatus::new(
            inner_status.max_size,
            inner_status.size,
            inner_status.available,
            inner_status.waiting,
        )
    }

    pub fn resize(&self, new_max_size: usize) {
        self.pool.resize(new_max_size);
    }

    /// Execute querystring with parameters.
    ///
    /// Prepare statement and cache it, then execute.
    ///
    /// # Errors
    /// May return Err Result if cannot retrieve new connection
    /// or prepare statement or execute statement.
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn execute<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_pool = pyo3::Python::with_gil(|gil| self_.borrow(gil).pool.clone());

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
    #[pyo3(signature = (querystring, parameters=None, prepared=None))]
    pub async fn fetch<'a>(
        self_: pyo3::Py<Self>,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let db_pool = pyo3::Python::with_gil(|gil| self_.borrow(gil).pool.clone());

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
        Connection::new(None, Some(self.pool.clone()))
    }

    pub async fn add_listener(
        self_: pyo3::Py<Self>,
        callback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<Listener> {
        let (pg_config, ca_file, ssl_mode) = pyo3::Python::with_gil(|gil| {
            let b_gil = self_.borrow(gil);
            (
                b_gil.pg_config.clone(),
                b_gil.ca_file.clone(),
                b_gil.ssl_mode,
            )
        });

        // let tls_ = build_tls(&ca_file, Some(SslMode::Disable)).unwrap();

        // match tls_ {
        //     ConfiguredTLS::NoTls => {
        //         let a = pg_config.connect(NoTls).await.unwrap();
        //     },
        //     ConfiguredTLS::TlsConnector(connector) => {
        //         let a = pg_config.connect(connector).await.unwrap();
        //     }
        // }

        // let (client, mut connection) = tokio_runtime()
        //     .spawn(async move { pg_config.connect(NoTls).await.unwrap() })
        //     .await?;

        // // Make transmitter and receiver.
        // let (tx, mut rx) = futures_channel::mpsc::unbounded();
        // let stream =
        //     stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!("{}", e));
        // let connection = stream.forward(tx).map(|r| r.unwrap());
        // tokio_runtime().spawn(connection);

        // // Wait for notifications in separate thread.
        // tokio_runtime().spawn(async move {
        //     client
        //         .batch_execute(
        //             "LISTEN test_notifications;
        //             LISTEN test_notifications2;",
        //         )
        //         .await
        //         .unwrap();

        //     loop {
        //         let next_element = rx.next().await;
        //         client.batch_execute("LISTEN test_notifications3;").await.unwrap();
        //         match next_element {
        //                 Some(n) => {
        //                     match n {
        //                         tokio_postgres::AsyncMessage::Notification(n) => {
        //                             Python::with_gil(|gil| {
        //                                 callback.call0(gil);
        //                             });
        //                             println!("Notification {:?}", n);
        //                         },
        //                         _ => {println!("in_in {:?}", n)}
        //                     }
        //                 },
        //                 _ => {println!("in {:?}", next_element)}
        //         }
        //     }
        //     });

        Ok(Listener::new(pg_config, ca_file, ssl_mode))
    }

    /// Return new single connection.
    ///
    /// # Errors
    /// May return Err Result if cannot get new connection from the pool.
    pub async fn connection(self_: pyo3::Py<Self>) -> RustPSQLDriverPyResult<Connection> {
        let db_pool = pyo3::Python::with_gil(|gil| self_.borrow(gil).pool.clone());
        let db_connection = tokio_runtime()
            .spawn(async move {
                Ok::<deadpool_postgres::Object, RustPSQLDriverError>(db_pool.get().await?)
            })
            .await??;

        Ok(Connection::new(Some(Arc::new(db_connection)), None))
    }

    /// Close connection pool.
    ///
    /// # Errors
    /// May return Err Result if cannot get new connection from the pool.
    pub fn close(&self) {
        let db_pool = self.pool.clone();

        db_pool.close();
    }
}
