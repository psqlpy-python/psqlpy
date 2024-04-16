use crate::runtime::tokio_runtime;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use pyo3::{pyclass, pyfunction, pymethods, PyAny};
use std::{str::FromStr, vec};
use tokio_postgres::{NoTls, Row};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO, QueryParameter},
};

use super::{common_options::ConnRecyclingMethod, connection::Connection};

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
    port: Option<u16>,
    db_name: Option<String>,
    max_db_pool_size: Option<usize>,
    conn_recycling_method: Option<ConnRecyclingMethod>,
) -> RustPSQLDriverPyResult<ConnectionPool> {
    if let Some(max_db_pool_size) = max_db_pool_size {
        if max_db_pool_size < 2 {
            return Err(RustPSQLDriverError::DataBasePoolConfigurationError(
                "Maximum database pool size must be more than 1".into(),
            ));
        }
    }

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
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);

    let mut db_pool_builder = Pool::builder(mgr);
    if let Some(max_db_pool_size) = max_db_pool_size {
        db_pool_builder = db_pool_builder.max_size(max_db_pool_size);
    }

    let db_pool = db_pool_builder.build()?;

    Ok(ConnectionPool(db_pool))
}

#[pyclass]
pub struct ConnectionPool(Pool);

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
        port: Option<u16>,
        db_name: Option<String>,
        max_db_pool_size: Option<usize>,
        conn_recycling_method: Option<ConnRecyclingMethod>,
    ) -> RustPSQLDriverPyResult<Self> {
        connect(
            dsn,
            username,
            password,
            host,
            port,
            db_name,
            max_db_pool_size,
            conn_recycling_method,
        )
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
        prepared: Option<bool>,
        parameters: Option<pyo3::Py<PyAny>>,
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
                    Ok::<Vec<Row>, RustPSQLDriverError>(
                        db_pool_manager
                            .query(
                                &db_pool_manager.prepare_cached(&querystring).await?,
                                &params
                                    .iter()
                                    .map(|param| param as &QueryParameter)
                                    .collect::<Vec<&QueryParameter>>()
                                    .into_boxed_slice(),
                            )
                            .await?,
                    )
                })
                .await??
        } else {
            tokio_runtime()
                .spawn(async move {
                    Ok::<Vec<Row>, RustPSQLDriverError>(
                        db_pool_manager
                            .query(
                                &querystring,
                                &params
                                    .iter()
                                    .map(|param| param as &QueryParameter)
                                    .collect::<Vec<&QueryParameter>>()
                                    .into_boxed_slice(),
                            )
                            .await?,
                    )
                })
                .await??
        };
        Ok(PSQLDriverPyQueryResult::new(result))
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

        Ok(Connection::new(db_connection))
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
