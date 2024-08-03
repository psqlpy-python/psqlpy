use std::{str::FromStr, time::Duration};

use crate::exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult};

use super::common_options::{LoadBalanceHosts, SslMode, TargetSessionAttrs};

/// Create new config.
///
/// # Errors
/// May return Err Result if cannot build new config.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn build_connection_config(
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
) -> RustPSQLDriverPyResult<tokio_postgres::Config> {
    if tcp_user_timeout_nanosec.is_some() && tcp_user_timeout_sec.is_none() {
        return Err(RustPSQLDriverError::ConnectionPoolConfigurationError(
            "tcp_user_timeout_nanosec must be used with tcp_user_timeout_sec param.".into(),
        ));
    }

    if connect_timeout_nanosec.is_some() && connect_timeout_sec.is_none() {
        return Err(RustPSQLDriverError::ConnectionPoolConfigurationError(
            "connect_timeout_nanosec must be used with connect_timeout_sec param.".into(),
        ));
    }

    if keepalives_idle_nanosec.is_some() && keepalives_idle_sec.is_none() {
        return Err(RustPSQLDriverError::ConnectionPoolConfigurationError(
            "keepalives_idle_nanosec must be used with keepalives_idle_sec param.".into(),
        ));
    }

    if keepalives_interval_nanosec.is_some() && keepalives_interval_sec.is_none() {
        return Err(RustPSQLDriverError::ConnectionPoolConfigurationError(
            "keepalives_interval_nanosec must be used with keepalives_interval_sec param.".into(),
        ));
    }

    let mut pg_config: tokio_postgres::Config;

    if let Some(dsn_string) = dsn {
        pg_config = tokio_postgres::Config::from_str(&dsn_string).map_err(|err| {
            RustPSQLDriverError::ConnectionPoolBuildError(format!(
                "Cannot parse configuration from dsn string, error - {err}"
            ))
        })?;
    } else {
        pg_config = tokio_postgres::Config::new();

        if let Some(password) = password {
            pg_config.password(&password);
        }

        if let Some(username) = username {
            pg_config.user(&username);
        }

        if let Some(hosts) = hosts {
            for single_host in hosts {
                pg_config.host(&single_host);
            }
        }

        if let Some(host) = host {
            pg_config.host(&host);
        }

        if let Some(ports) = ports {
            for single_port in ports {
                pg_config.port(single_port);
            }
        }

        if let Some(port) = port {
            pg_config.port(port);
        }

        if let Some(db_name) = db_name {
            pg_config.dbname(&db_name);
        }

        if let Some(target_session_attrs) = target_session_attrs {
            pg_config.target_session_attrs(target_session_attrs.to_internal());
        }
    }

    if let Some(options) = options {
        pg_config.options(&options);
    }

    if let Some(application_name) = application_name {
        pg_config.application_name(&application_name);
    }

    if let Some(connect_timeout_sec) = connect_timeout_sec {
        pg_config.connect_timeout(Duration::new(
            connect_timeout_sec,
            connect_timeout_nanosec.unwrap_or_default(),
        ));
    }

    if let Some(tcp_user_timeout_sec) = tcp_user_timeout_sec {
        pg_config.tcp_user_timeout(Duration::new(
            tcp_user_timeout_sec,
            tcp_user_timeout_nanosec.unwrap_or_default(),
        ));
    }

    if let Some(keepalives) = keepalives {
        if keepalives {
            pg_config.keepalives(keepalives);

            if let Some(keepalives_idle_sec) = keepalives_idle_sec {
                pg_config.keepalives_idle(Duration::new(
                    keepalives_idle_sec,
                    keepalives_idle_nanosec.unwrap_or_default(),
                ));
            }

            if let Some(keepalives_interval_sec) = keepalives_interval_sec {
                pg_config.keepalives_interval(Duration::new(
                    keepalives_interval_sec,
                    keepalives_interval_nanosec.unwrap_or_default(),
                ));
            }

            if let Some(keepalives_retries) = keepalives_retries {
                pg_config.keepalives_retries(keepalives_retries);
            }
        }
    }

    if let Some(load_balance_hosts) = load_balance_hosts {
        pg_config.load_balance_hosts(load_balance_hosts.to_internal());
    }

    if let Some(ssl_mode) = ssl_mode {
        pg_config.ssl_mode(ssl_mode.to_internal());
    }

    Ok(pg_config)
}
