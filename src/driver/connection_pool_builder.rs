use std::{net::IpAddr, time::Duration};

use pyo3::{pyclass, pymethods, Py, Python};
use tokio_postgres::config::{ChannelBinding, Host, LoadBalanceHosts, SslMode, TargetSessionAttrs};

#[pyclass]
pub struct ConnectionPoolBuilder {
    pub user: Option<String>,
    pub password: Option<Vec<u8>>,
    pub dbname: Option<String>,
    pub options: Option<String>,
    pub application_name: Option<String>,
    pub ssl_mode: SslMode,
    pub host: Vec<Host>,
    pub hostaddr: Vec<IpAddr>,
    pub port: Vec<u16>,
    pub connect_timeout: Option<Duration>,
    pub tcp_user_timeout: Option<Duration>,
    pub keepalives: bool,
    #[cfg(not(target_arch = "wasm32"))]
    // pub keepalive_config: KeepaliveConfig,
    pub target_session_attrs: TargetSessionAttrs,
    pub channel_binding: ChannelBinding,
    pub load_balance_hosts: LoadBalanceHosts,
}

#[pymethods]
impl ConnectionPoolBuilder {
    #[new]
    fn new() -> Self {
        ConnectionPoolBuilder {
            user: None,
            password: None,
            dbname: None,
            options: None,
            application_name: None,
            ssl_mode: SslMode::Prefer,
            host: vec![],
            hostaddr: vec![],
            port: vec![],
            connect_timeout: None,
            tcp_user_timeout: None,
            keepalives: true,
            // #[cfg(not(target_arch = "wasm32"))]
            // keepalive_config: KeepaliveConfig {
            //     idle: Duration::from_secs(2 * 60 * 60),
            //     interval: None,
            //     retries: None,
            // },
            target_session_attrs: TargetSessionAttrs::Any,
            channel_binding: ChannelBinding::Prefer,
            load_balance_hosts: LoadBalanceHosts::Disable,
        }
    }

    /// Sets the user to authenticate with.
    ///
    /// Defaults to the user executing this process.
    #[must_use]
    pub fn user(self_: Py<Self>, user: String) -> Py<ConnectionPoolBuilder> {
        Python::with_gil(|gil| {
            let mut self_ = self_.borrow_mut(gil);
            self_.user = Some(user);
        });
        self_
    }

    // /// Sets the password to authenticate with.
    // pub fn password(&mut self, password: String) -> &mut ConnectionPoolBuilder {
    //     self.password = Some(password.into_bytes().to_vec());
    //     self
    // }

    // /// Gets the password to authenticate with, if one has been configured with
    // /// the `password` method.
    // pub fn get_password(&self) -> Option<&[u8]> {
    //     self.password.as_deref()
    // }

    // /// Sets the name of the database to connect to.
    // ///
    // /// Defaults to the user.
    // pub fn dbname(&mut self, dbname: &str) -> &mut ConnectionPoolBuilder {
    //     self.dbname = Some(dbname.to_string());
    //     self
    // }

    // /// Gets the name of the database to connect to, if one has been configured
    // /// with the `dbname` method.
    // pub fn get_dbname(&self) -> Option<&str> {
    //     self.dbname.as_deref()
    // }

    // /// Sets command line options used to configure the server.
    // pub fn options(&mut self, options: &str) -> &mut ConnectionPoolBuilder {
    //     self.options = Some(options.to_string());
    //     self
    // }

    // /// Gets the command line options used to configure the server, if the
    // /// options have been set with the `options` method.
    // pub fn get_options(&self) -> Option<&str> {
    //     self.options.as_deref()
    // }

    // /// Sets the value of the `application_name` runtime parameter.
    // pub fn application_name(&mut self, application_name: &str) -> &mut ConnectionPoolBuilder {
    //     self.application_name = Some(application_name.to_string());
    //     self
    // }

    // /// Gets the value of the `application_name` runtime parameter, if it has
    // /// been set with the `application_name` method.
    // pub fn get_application_name(&self) -> Option<&str> {
    //     self.application_name.as_deref()
    // }

    // /// Sets the SSL configuration.
    // ///
    // /// Defaults to `prefer`.
    // pub fn ssl_mode(&mut self, ssl_mode: SslMode) -> &mut ConnectionPoolBuilder {
    //     self.ssl_mode = ssl_mode;
    //     self
    // }

    // /// Gets the SSL configuration.
    // pub fn get_ssl_mode(&self) -> SslMode {
    //     self.ssl_mode
    // }

    // /// Adds a host to the configuration.
    // ///
    // /// Multiple hosts can be specified by calling this method multiple times, and each will be tried in order. On Unix
    // /// systems, a host starting with a `/` is interpreted as a path to a directory containing Unix domain sockets.
    // /// There must be either no hosts, or the same number of hosts as hostaddrs.
    // pub fn host(&mut self, host: &str) -> &mut ConnectionPoolBuilder {
    //     #[cfg(unix)]
    //     {
    //         if host.starts_with('/') {
    //             return self.host_path(host);
    //         }
    //     }

    //     self.host.push(Host::Tcp(host.to_string()));
    //     self
    // }

    // /// Gets the hosts that have been added to the configuration with `host`.
    // pub fn get_hosts(&self) -> &[Host] {
    //     &self.host
    // }

    // /// Adds a Unix socket host to the configuration.
    // ///
    // /// Unlike `host`, this method allows non-UTF8 paths.
    // #[cfg(unix)]
    // pub fn host_path(&mut self, host: String) -> &mut ConnectionPoolBuilder {
    //     self.host.push(Host::Unix(host.bytes().to_path_buf()));
    //     self
    // }

    // /// Adds a hostaddr to the configuration.
    // ///
    // /// Multiple hostaddrs can be specified by calling this method multiple times, and each will be tried in order.
    // /// There must be either no hostaddrs, or the same number of hostaddrs as hosts.
    // pub fn hostaddr(&mut self, hostaddr: IpAddr) -> &mut ConnectionPoolBuilder {
    //     self.hostaddr.push(hostaddr);
    //     self
    // }

    // /// Adds a port to the configuration.
    // ///
    // /// Multiple ports can be specified by calling this method multiple times. There must either be no ports, in which
    // /// case the default of 5432 is used, a single port, in which it is used for all hosts, or the same number of ports
    // /// as hosts.
    // pub fn port(&mut self, port: u16) -> &mut ConnectionPoolBuilder {
    //     self.port.push(port);
    //     self
    // }

    // /// Gets the ports that have been added to the configuration with `port`.
    // pub fn get_ports(&self) -> &[u16] {
    //     &self.port
    // }

    // /// Sets the timeout applied to socket-level connection attempts.
    // ///
    // /// Note that hostnames can resolve to multiple IP addresses, and this timeout will apply to each address of each
    // /// host separately. Defaults to no limit.
    // pub fn connect_timeout(&mut self, connect_timeout: Duration) -> &mut ConnectionPoolBuilder {
    //     self.connect_timeout = Some(connect_timeout);
    //     self
    // }

    // /// Gets the connection timeout, if one has been set with the
    // /// `connect_timeout` method.
    // pub fn get_connect_timeout(&self) -> Option<&Duration> {
    //     self.connect_timeout.as_ref()
    // }

    // /// Sets the TCP user timeout.
    // ///
    // /// This is ignored for Unix domain socket connections. It is only supported on systems where
    // /// TCP_USER_TIMEOUT is available and will default to the system default if omitted or set to 0;
    // /// on other systems, it has no effect.
    // pub fn tcp_user_timeout(&mut self, tcp_user_timeout: Duration) -> &mut ConnectionPoolBuilder {
    //     self.tcp_user_timeout = Some(tcp_user_timeout);
    //     self
    // }

    // /// Gets the TCP user timeout, if one has been set with the
    // /// `user_timeout` method.
    // pub fn get_tcp_user_timeout(&self) -> Option<&Duration> {
    //     self.tcp_user_timeout.as_ref()
    // }

    // /// Controls the use of TCP keepalive.
    // ///
    // /// This is ignored for Unix domain socket connections. Defaults to `true`.
    // pub fn keepalives(&mut self, keepalives: bool) -> &mut ConnectionPoolBuilder {
    //     self.keepalives = keepalives;
    //     self
    // }

    // /// Reports whether TCP keepalives will be used.
    // pub fn get_keepalives(&self) -> bool {
    //     self.keepalives
    // }

    // // /// Sets the amount of idle time before a keepalive packet is sent on the connection.
    // // ///
    // // /// This is ignored for Unix domain sockets, or if the `keepalives` option is disabled. Defaults to 2 hours.
    // // #[cfg(not(target_arch = "wasm32"))]
    // // pub fn keepalives_idle(&mut self, keepalives_idle: Duration) -> &mut ConnectionPoolBuilder {
    // //     self.keepalive_config.idle = keepalives_idle;
    // //     self
    // // }

    // // /// Gets the configured amount of idle time before a keepalive packet will
    // // /// be sent on the connection.
    // // #[cfg(not(target_arch = "wasm32"))]
    // // pub fn get_keepalives_idle(&self) -> Duration {
    // //     self.keepalive_config.idle
    // // }

    // // /// Sets the time interval between TCP keepalive probes.
    // // /// On Windows, this sets the value of the tcp_keepalive struct’s keepaliveinterval field.
    // // ///
    // // /// This is ignored for Unix domain sockets, or if the `keepalives` option is disabled.
    // // #[cfg(not(target_arch = "wasm32"))]
    // // pub fn keepalives_interval(
    // //     &mut self,
    // //     keepalives_interval: Duration,
    // // ) -> &mut ConnectionPoolBuilder {
    // //     self.keepalive_config.interval = Some(keepalives_interval);
    // //     self
    // // }

    // // /// Gets the time interval between TCP keepalive probes.
    // // #[cfg(not(target_arch = "wasm32"))]
    // // pub fn get_keepalives_interval(&self) -> Option<Duration> {
    // //     self.keepalive_config.interval
    // // }

    // // /// Sets the maximum number of TCP keepalive probes that will be sent before dropping a connection.
    // // ///
    // // /// This is ignored for Unix domain sockets, or if the `keepalives` option is disabled.
    // // #[cfg(not(target_arch = "wasm32"))]
    // // pub fn keepalives_retries(&mut self, keepalives_retries: u32) -> &mut ConnectionPoolBuilder {
    // //     self.keepalive_config.retries = Some(keepalives_retries);
    // //     self
    // // }

    // // /// Gets the maximum number of TCP keepalive probes that will be sent before dropping a connection.
    // // #[cfg(not(target_arch = "wasm32"))]
    // // pub fn get_keepalives_retries(&self) -> Option<u32> {
    // //     self.keepalive_config.retries
    // // }

    // /// Sets the requirements of the session.
    // ///
    // /// This can be used to connect to the primary server in a clustered database rather than one of the read-only
    // /// secondary servers. Defaults to `Any`.
    // pub fn target_session_attrs(
    //     &mut self,
    //     target_session_attrs: TargetSessionAttrs,
    // ) -> &mut ConnectionPoolBuilder {
    //     self.target_session_attrs = target_session_attrs;
    //     self
    // }

    // /// Gets the requirements of the session.
    // pub fn get_target_session_attrs(&self) -> TargetSessionAttrs {
    //     self.target_session_attrs
    // }

    // /// Sets the channel binding behavior.
    // ///
    // /// Defaults to `prefer`.
    // pub fn channel_binding(
    //     &mut self,
    //     channel_binding: ChannelBinding,
    // ) -> &mut ConnectionPoolBuilder {
    //     self.channel_binding = channel_binding;
    //     self
    // }

    // /// Gets the channel binding behavior.
    // pub fn get_channel_binding(&self) -> ChannelBinding {
    //     self.channel_binding
    // }

    // /// Sets the host load balancing behavior.
    // ///
    // /// Defaults to `disable`.
    // pub fn load_balance_hosts(
    //     &mut self,
    //     load_balance_hosts: LoadBalanceHosts,
    // ) -> &mut ConnectionPoolBuilder {
    //     self.load_balance_hosts = load_balance_hosts;
    //     self
    // }
}
