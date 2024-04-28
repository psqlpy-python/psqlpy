use std::{str::FromStr, time::Duration};

use crate::exceptions::rust_errors::RustPSQLDriverPyResult;

use super::common_options::ConnLoadBalanceHosts;

/// Create new config.
///
/// # Errors
/// May return Err Result if cannot build new config.
#[allow(clippy::too_many_arguments)]
pub fn build_connection_config(
    dsn: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    db_name: Option<String>,
    options: Option<String>,
    application_name: Option<String>,
    connect_timeout: Option<Duration>,
    tcp_user_timeout: Option<Duration>,
    keepalives: Option<bool>,
    keepalives_idle: Option<Duration>,
    keepalives_interval: Option<Duration>,
    keepalives_retries: Option<u32>,
    load_balance_hosts: Option<ConnLoadBalanceHosts>,
) -> RustPSQLDriverPyResult<tokio_postgres::Config> {
    if let Some(dsn_string) = dsn {
        return Ok(tokio_postgres::Config::from_str(&dsn_string)?);
    }

    let mut pg_config = tokio_postgres::Config::new();

    if let (Some(password), Some(username)) = (password, username) {
        pg_config.password(&password);
        pg_config.user(&username);
    }
    if let Some(host) = host {
        pg_config.host(&host);
    }

    if let Some(port) = port {
        pg_config.port(port);
    }

    if let Some(db_name) = db_name {
        pg_config.dbname(&db_name);
    }

    if let Some(options) = options {
        pg_config.options(&options);
    }

    if let Some(application_name) = application_name {
        pg_config.application_name(&application_name);
    }

    if let Some(connect_timeout) = connect_timeout {
        pg_config.connect_timeout(connect_timeout);
    }

    if let Some(tcp_user_timeout) = tcp_user_timeout {
        pg_config.tcp_user_timeout(tcp_user_timeout);
    }

    if let Some(keepalives) = keepalives {
        if keepalives {
            pg_config.keepalives(keepalives);

            if let Some(keepalives_idle) = keepalives_idle {
                pg_config.keepalives_idle(keepalives_idle);
            }

            if let Some(keepalives_interval) = keepalives_interval {
                pg_config.keepalives_interval(keepalives_interval);
            }

            if let Some(keepalives_retries) = keepalives_retries {
                pg_config.keepalives_retries(keepalives_retries);
            }
        }
    }

    if let Some(load_balance_hosts) = load_balance_hosts {
        pg_config.load_balance_hosts(load_balance_hosts.to_internal());
    }

    Ok(pg_config)
}
