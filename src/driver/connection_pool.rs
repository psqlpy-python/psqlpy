use crate::runtime::tokio_runtime;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use pyo3::{pyclass, pymethods, PyAny};
use std::{str::FromStr, vec};
use tokio_postgres::{NoTls, Row};

use crate::{
    // common::rustdriver_future,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    query_result::PSQLDriverPyQueryResult,
    value_converter::{convert_parameters, PythonDTO, QueryParameter},
};

use super::{
    common_options::ConnRecyclingMethod,
    connection::Connection,
    // connection::{Connection, RustConnection},
};

/// `PSQLPool` is for internal use only.
///
/// It is not exposed to python.
// pub struct RustPSQLPool {
//     dsn: Option<String>,
//     username: Option<String>,
//     password: Option<String>,
//     host: Option<String>,
//     port: Option<u16>,
//     db_name: Option<String>,
//     max_db_pool_size: Option<usize>,
//     conn_recycling_method: Option<ConnRecyclingMethod>,
//     db_pool: Option<Pool>,
// }

// impl RustPSQLPool {
//     /// Create new `RustPSQLPool`.
//     #[must_use]
//     #[allow(clippy::too_many_arguments)]
//     pub fn new(
//         dsn: Option<String>,
//         username: Option<String>,
//         password: Option<String>,
//         host: Option<String>,
//         port: Option<u16>,
//         db_name: Option<String>,
//         max_db_pool_size: Option<usize>,
//         conn_recycling_method: Option<ConnRecyclingMethod>,
//     ) -> Self {
//         RustPSQLPool {
//             dsn,
//             username,
//             password,
//             host,
//             port,
//             db_name,
//             max_db_pool_size,
//             conn_recycling_method,
//             db_pool: None,
//         }
//     }
// }

// impl RustPSQLPool {
//     /// Return new single connection.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot get new connection from the pool.
//     pub async fn inner_connection(&self) -> RustPSQLDriverPyResult<Connection> {
//         let db_pool_manager = self
//             .db_pool
//             .as_ref()
//             .ok_or(RustPSQLDriverError::DatabasePoolError(
//                 "Database pool is not initialized".into(),
//             ))?
//             .get()
//             .await?;

//         Ok(Connection::new(Arc::new(RustConnection::new(Arc::new(
//             tokio::sync::RwLock::new(db_pool_manager),
//         )))))
//     }
//     /// Execute querystring with parameters.
//     ///
//     /// Prepare statement and cache it, then execute.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot retrieve new connection
//     /// or prepare statement or execute statement.
//     pub async fn inner_execute(
//         &self,
//         querystring: String,
//         parameters: Vec<PythonDTO>,
//         prepared: bool,
//     ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
//         let db_pool_manager = self
//             .db_pool
//             .as_ref()
//             .ok_or(RustPSQLDriverError::DatabasePoolError(
//                 "Database pool is not initialized".into(),
//             ))?
//             .get()
//             .await?;

//         let vec_parameters: Vec<&QueryParameter> = parameters
//             .iter()
//             .map(|param| param as &QueryParameter)
//             .collect();

//         let result = if prepared {
//             db_pool_manager
//                 .query(
//                     &db_pool_manager.prepare_cached(&querystring).await?,
//                     &vec_parameters.into_boxed_slice(),
//                 )
//                 .await?
//         } else {
//             db_pool_manager
//                 .query(&querystring, &vec_parameters.into_boxed_slice())
//                 .await?
//         };
//         Ok(PSQLDriverPyQueryResult::new(result))
//     }

//     /// Create new Database pool.
//     ///
//     /// # Errors
//     /// May return Err Result if Database pool is already initialized,
//     /// `max_db_pool_size` is less than 2 or it's impossible to build db pool.
//     pub fn inner_startup(mut self) -> RustPSQLDriverPyResult<Self> {
//         let dsn = self.dsn.clone();
//         let password = self.password.clone();
//         let username = self.username.clone();
//         let db_host = self.host.clone();
//         let db_port = self.port;
//         let db_name = self.db_name.clone();
//         let conn_recycling_method = self.conn_recycling_method;
//         let max_db_pool_size = self.max_db_pool_size;

//         if self.db_pool.is_some() {
//             return Err(RustPSQLDriverError::DatabasePoolError(
//                 "Database pool is already initialized".into(),
//             ));
//         }

//         if let Some(max_db_pool_size) = max_db_pool_size {
//             if max_db_pool_size < 2 {
//                 return Err(RustPSQLDriverError::DataBasePoolConfigurationError(
//                     "Maximum database pool size must be more than 1".into(),
//                 ));
//             }
//         }

//         let mut pg_config: tokio_postgres::Config;
//         if let Some(dsn_string) = dsn {
//             pg_config = tokio_postgres::Config::from_str(&dsn_string)?;
//         } else {
//             pg_config = tokio_postgres::Config::new();
//             if let (Some(password), Some(username)) = (password, username) {
//                 pg_config.password(&password);
//                 pg_config.user(&username);
//             }
//             if let Some(db_host) = db_host {
//                 pg_config.host(&db_host);
//             }

//             if let Some(db_port) = db_port {
//                 pg_config.port(db_port);
//             }

//             if let Some(db_name) = db_name {
//                 pg_config.dbname(&db_name);
//             }
//         }

//         let mgr_config: ManagerConfig;
//         if let Some(conn_recycling_method) = conn_recycling_method {
//             mgr_config = ManagerConfig {
//                 recycling_method: conn_recycling_method.to_internal(),
//             }
//         } else {
//             mgr_config = ManagerConfig {
//                 recycling_method: RecyclingMethod::Fast,
//             };
//         }
//         let mgr = Manager::from_config(pg_config, NoTls, mgr_config);

//         let mut db_pool_builder = Pool::builder(mgr);
//         if let Some(max_db_pool_size) = max_db_pool_size {
//             db_pool_builder = db_pool_builder.max_size(max_db_pool_size);
//         }

//         self.db_pool = Some(db_pool_builder.build()?);
//         Ok(self)
//     }

//     /// Close connection pool.
//     ///
//     /// # Errors
//     /// May return Err Result if connection pool isn't opened.
//     pub fn inner_close(&self) -> RustPSQLDriverPyResult<()> {
//         let db_pool_manager =
//             self.db_pool
//                 .as_ref()
//                 .ok_or(RustPSQLDriverError::DatabasePoolError(
//                     "Database pool is not initialized".into(),
//                 ))?;

//         db_pool_manager.close();

//         Ok(())
//     }
// }

// #[pyclass()]
// pub struct PSQLPool {
//     rust_psql_pool: Arc<RustPSQLPool>,
// }

// #[pymethods]
// impl PSQLPool {
//     #[new]
//     #[allow(clippy::too_many_arguments)]
//     #[allow(clippy::missing_errors_doc)]
//     pub fn new(
//         dsn: Option<String>,
//         username: Option<String>,
//         password: Option<String>,
//         host: Option<String>,
//         port: Option<u16>,
//         db_name: Option<String>,
//         max_db_pool_size: Option<usize>,
//         conn_recycling_method: Option<ConnRecyclingMethod>,
//     ) -> RustPSQLDriverPyResult<Self> {
//         let inner_pool = RustPSQLPool {
//             dsn,
//             username,
//             password,
//             host,
//             port,
//             db_name,
//             max_db_pool_size,
//             conn_recycling_method,
//             db_pool: None,
//         }
//         .inner_startup()?;
//         Ok(PSQLPool {
//             rust_psql_pool: Arc::new(inner_pool),
//         })
//     }

//     /// Return single connection.
//     ///
//     /// # Errors
//     /// May return Err Result if `inner_connection` returns error.
//     pub fn connection<'a>(&'a self, py: Python<'a>) -> RustPSQLDriverPyResult<&'a PyAny> {
//         let psql_pool_arc = self.rust_psql_pool.clone();

//         rustdriver_future(py, async move { psql_pool_arc.inner_connection().await })
//     }

//     /// Execute querystring with parameters.
//     ///
//     /// # Errors
//     /// May return Err Result if cannot convert parameters
//     /// or `inner_execute` returns Err.
//     pub fn execute<'a>(
//         &'a self,
//         py: Python<'a>,
//         querystring: String,
//         parameters: Option<&'a PyAny>,
//         prepared: Option<bool>,
//     ) -> RustPSQLDriverPyResult<&'a PyAny> {
//         let db_pool_arc = self.rust_psql_pool.clone();
//         let mut params: Vec<PythonDTO> = vec![];
//         if let Some(parameters) = parameters {
//             params = convert_parameters(parameters)?;
//         }

//         rustdriver_future(py, async move {
//             // let db_pool_guard = db_pool_arc.read().await;
//             db_pool_arc
//                 .inner_execute(querystring, params, prepared.unwrap_or(true))
//                 .await
//         })
//     }

// Close connection pool.
//
// # Errors
//May return Err Result if connection pool isn't opened.
// pub fn close<'a>(&self, py: Python<'a>) -> RustPSQLDriverPyResult<&'a PyAny> {
//     let db_pool_arc = self.rust_psql_pool.clone();

//     rustdriver_future(py, async move {
//         // let db_pool_guard = db_pool_arc.read().await;

//         db_pool_arc.inner_close()
//     })
// }
// }

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
