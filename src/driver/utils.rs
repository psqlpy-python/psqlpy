use std::str::FromStr;

use crate::exceptions::rust_errors::RustPSQLDriverPyResult;

/// Build new config for making connection pool or single connection
///
/// # Errors
/// May return Err Result if cannot build config from dsn.
pub fn build_connection_config(
    dsn: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    db_name: Option<String>,
) -> RustPSQLDriverPyResult<tokio_postgres::Config> {
    let mut pg_config: tokio_postgres::Config;
    if let Some(dsn_string) = dsn {
        pg_config = tokio_postgres::Config::from_str(&dsn_string)?;
    } else {
        pg_config = tokio_postgres::Config::new();
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
    }

    Ok(pg_config)
}
