use std::{net::IpAddr, time::Duration};

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use pyo3::{pyclass, pymethods, Py, Python};
use tokio_postgres::NoTls;

use crate::exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult};

use super::connection_pool::ConnectionPool;

#[pyclass]
pub struct ConnectionPoolBuilder {
    config: tokio_postgres::Config,
    max_db_pool_size: Option<usize>,
    conn_recycling_method: Option<RecyclingMethod>,
    ca_file: Option<String>,
}

#[pymethods]
impl ConnectionPoolBuilder {
    /// Create new connection pool builder.
    #[new]
    fn new() -> Self {
        ConnectionPoolBuilder {
            config: tokio_postgres::Config::new(),
            max_db_pool_size: Some(2),
            conn_recycling_method: None,
            ca_file: None,
        }
    }

    /// Build connection pool.
    ///
    /// # Errors
    /// May return error if cannot build new connection pool.
    fn build(&self) -> RustPSQLDriverPyResult<ConnectionPool> {
        let mgr_config: ManagerConfig;
        if let Some(conn_recycling_method) = self.conn_recycling_method.as_ref() {
            mgr_config = ManagerConfig {
                recycling_method: conn_recycling_method.clone(),
            }
        } else {
            mgr_config = ManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            };
        };

        let mgr: Manager;
        if let Some(ca_file) = &self.ca_file {
            let mut builder = SslConnector::builder(SslMethod::tls())?;
            builder.set_ca_file(ca_file)?;
            let tls_connector = MakeTlsConnector::new(builder.build());
            mgr = Manager::from_config(self.config.clone(), tls_connector, mgr_config);
        } else {
            mgr = Manager::from_config(self.config.clone(), NoTls, mgr_config);
        }

        let mut db_pool_builder = Pool::builder(mgr);
        if let Some(max_db_pool_size) = self.max_db_pool_size {
            db_pool_builder = db_pool_builder.max_size(max_db_pool_size);
        }

        let db_pool = db_pool_builder.build()?;

        Ok(ConnectionPool(db_pool))
    }

    /// Set ca_file for ssl_mode in PostgreSQL.
    fn ca_file(self_: Py<Self>, ca_file: String) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.ca_file = Some(ca_file);
        });
        self_
    }

    /// Set size to the connection pool.
    ///
    /// # Error
    /// If size more than 2.
    fn max_pool_size(self_: Py<Self>, pool_size: usize) -> RustPSQLDriverPyResult<Py<Self>> {
        if pool_size < 2 {
            return Err(RustPSQLDriverError::ConnectionPoolConfigurationError(
                "Maximum database pool size must be more than 1".into(),
            ));
        }

        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.max_db_pool_size = Some(pool_size);
        });
        Ok(self_)
    }

    /// Set connection recycling method.
    fn conn_recycling_method(
        self_: Py<Self>,
        conn_recycling_method: super::common_options::ConnRecyclingMethod,
    ) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.conn_recycling_method = Some(conn_recycling_method.to_internal());
        });
        self_
    }

    /// Sets the user to authenticate with.
    ///
    /// Defaults to the user executing this process.
    #[must_use]
    pub fn user(self_: Py<Self>, user: &str) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.user(user);
        });
        self_
    }

    /// Sets the password to authenticate with.
    #[must_use]
    pub fn password(self_: Py<Self>, password: &str) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.password(password);
        });
        self_
    }

    /// Sets the name of the database to connect to.
    ///
    /// Defaults to the user.
    #[must_use]
    pub fn dbname(self_: Py<Self>, dbname: &str) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.dbname(dbname);
        });
        self_
    }

    /// Sets command line options used to configure the server.
    #[must_use]
    pub fn options(self_: Py<Self>, options: &str) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.options(options);
        });
        self_
    }

    /// Sets the value of the `application_name` runtime parameter.
    #[must_use]
    pub fn application_name(self_: Py<Self>, application_name: &str) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.application_name(application_name);
        });
        self_
    }

    /// Sets the SSL configuration.
    ///
    /// Defaults to `prefer`.
    #[must_use]
    pub fn ssl_mode(self_: Py<Self>, ssl_mode: crate::driver::common_options::SslMode) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.ssl_mode(ssl_mode.to_internal());
        });
        self_
    }

    /// Adds a host to the configuration.
    ///
    /// Multiple hosts can be specified by calling this method multiple times, and each will be tried in order. On Unix
    /// systems, a host starting with a `/` is interpreted as a path to a directory containing Unix domain sockets.
    /// There must be either no hosts, or the same number of hosts as hostaddrs.
    #[must_use]
    pub fn host(self_: Py<Self>, host: &str) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.host(host);
        });
        self_
    }

    /// Adds a hostaddr to the configuration.
    ///
    /// Multiple hostaddrs can be specified by calling this method multiple times, and each will be tried in order.
    /// There must be either no hostaddrs, or the same number of hostaddrs as hosts.
    #[must_use]
    pub fn hostaddr(self_: Py<Self>, hostaddr: IpAddr) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.hostaddr(hostaddr);
        });
        self_
    }

    /// Adds a port to the configuration.
    ///
    /// Multiple ports can be specified by calling this method multiple times. There must either be no ports, in which
    /// case the default of 5432 is used, a single port, in which it is used for all hosts, or the same number of ports
    /// as hosts.
    #[must_use]
    pub fn port(self_: Py<Self>, port: u16) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.port(port);
        });
        self_
    }

    /// Sets the timeout applied to socket-level connection attempts.
    ///
    /// Note that hostnames can resolve to multiple IP addresses, and this timeout will apply to each address of each
    /// host separately. Defaults to no limit.
    #[must_use]
    pub fn connect_timeout(self_: Py<Self>, connect_timeout: u64) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_
                .config
                .connect_timeout(Duration::from_secs(connect_timeout));
        });
        self_
    }

    /// Sets the TCP user timeout.
    ///
    /// This is ignored for Unix domain socket connections. It is only supported on systems where
    /// TCP_USER_TIMEOUT is available and will default to the system default if omitted or set to 0;
    /// on other systems, it has no effect.
    #[must_use]
    pub fn tcp_user_timeout(self_: Py<Self>, tcp_user_timeout: u64) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_
                .config
                .tcp_user_timeout(Duration::from_secs(tcp_user_timeout));
        });
        self_
    }

    /// Sets the requirements of the session.
    ///
    /// This can be used to connect to the primary server in a clustered database rather than one of the read-only
    /// secondary servers. Defaults to `Any`.
    #[must_use]
    pub fn target_session_attrs(
        self_: Py<Self>,
        target_session_attrs: super::common_options::TargetSessionAttrs,
    ) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_
                .config
                .target_session_attrs(target_session_attrs.to_internal());
        });
        self_
    }

    /// Sets the host load balancing behavior.
    ///
    /// Defaults to `disable`.
    #[must_use]
    pub fn load_balance_hosts(
        self_: Py<Self>,
        load_balance_hosts: super::common_options::LoadBalanceHosts,
    ) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_
                .config
                .load_balance_hosts(load_balance_hosts.to_internal());
        });
        self_
    }

    /// Controls the use of TCP keepalive.
    ///
    /// This is ignored for Unix domain socket connections. Defaults to `true`.
    #[must_use]
    pub fn keepalives(self_: Py<Self>, keepalives: bool) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.keepalives(keepalives);
        });
        self_
    }

    /// Sets the amount of idle time before a keepalive packet is sent on the connection.
    ///
    /// This is ignored for Unix domain sockets, or if the `keepalives` option is disabled. Defaults to 2 hours.
    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn keepalives_idle(self_: Py<Self>, keepalives_idle: u64) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_
                .config
                .keepalives_idle(Duration::from_secs(keepalives_idle));
        });
        self_
    }

    /// Sets the time interval between TCP keepalive probes.
    /// On Windows, this sets the value of the tcp_keepalive struct’s keepaliveinterval field.
    ///
    /// This is ignored for Unix domain sockets, or if the `keepalives` option is disabled.
    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn keepalives_interval(self_: Py<Self>, keepalives_interval: u64) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_
                .config
                .keepalives_interval(Duration::from_secs(keepalives_interval));
        });
        self_
    }

    /// Sets the maximum number of TCP keepalive probes that will be sent before dropping a connection.
    ///
    /// This is ignored for Unix domain sockets, or if the `keepalives` option is disabled.
    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn keepalives_retries(self_: Py<Self>, keepalives_retries: u32) -> Py<Self> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.config.keepalives_retries(keepalives_retries);
        });
        self_
    }
}
