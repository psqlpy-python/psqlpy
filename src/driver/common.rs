use tokio_postgres::config::Host;

use std::net::IpAddr;

use super::{
    connection::Connection, cursor::Cursor, prepared_statement::PreparedStatement,
    transaction::Transaction,
};

use pyo3::{pymethods, Py, PyAny};

use crate::{
    connection::traits::CloseTransaction,
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
};

use bytes::BytesMut;
use futures_util::pin_mut;
use pyo3::{buffer::PyBuffer, Python};
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use crate::format_helpers::quote_ident;

macro_rules! impl_config_py_methods {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            #[getter]
            fn conn_dbname(&self) -> Option<&str> {
                self.pg_config.get_dbname()
            }

            #[getter]
            fn user(&self) -> Option<&str> {
                self.pg_config.get_user()
            }

            #[getter]
            fn host_addrs(&self) -> Vec<String> {
                let mut host_addrs_vec = vec![];

                let host_addrs = self.pg_config.get_hostaddrs();
                for ip_addr in host_addrs {
                    match ip_addr {
                        IpAddr::V4(ipv4) => {
                            host_addrs_vec.push(ipv4.to_string());
                        }
                        IpAddr::V6(ipv6) => {
                            host_addrs_vec.push(ipv6.to_string());
                        }
                    }
                }

                host_addrs_vec
            }

            #[cfg(unix)]
            #[getter]
            fn hosts(&self) -> Vec<String> {
                let mut hosts_vec = vec![];

                let hosts = self.pg_config.get_hosts();
                for host in hosts {
                    match host {
                        Host::Tcp(host) => {
                            hosts_vec.push(host.to_string());
                        }
                        Host::Unix(host) => {
                            hosts_vec.push(host.display().to_string());
                        }
                    }
                }

                hosts_vec
            }

            #[cfg(not(unix))]
            #[getter]
            fn hosts(&self) -> Vec<String> {
                self.pg_config
                    .get_hosts()
                    .iter()
                    .map(|host| match host {
                        Host::Tcp(host) => host.to_string(),
                    })
                    .collect()
            }

            #[getter]
            fn ports(&self) -> Vec<&u16> {
                return self.pg_config.get_ports().iter().collect::<Vec<&u16>>();
            }

            #[getter]
            fn options(&self) -> Option<&str> {
                return self.pg_config.get_options();
            }
        }
    };
}

impl_config_py_methods!(Transaction);
impl_config_py_methods!(Connection);
impl_config_py_methods!(Cursor);
// impl_config_py_methods!(Portal);

macro_rules! impl_is_closed_method {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            fn is_closed(&self) -> bool {
                if self.conn.is_some() {
                    return true;
                }
                false
            }
        }
    };
}

impl_is_closed_method!(Transaction);
impl_is_closed_method!(Connection);
impl_is_closed_method!(Cursor);

macro_rules! impl_cursor_method {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            #[pyo3(signature = (querystring=None, parameters=None, array_size=None))]
            #[must_use]
            pub fn cursor(
                &self,
                querystring: Option<String>,
                parameters: Option<Py<PyAny>>,
                array_size: Option<i32>,
            ) -> Cursor {
                Cursor::new(
                    self.conn.clone(),
                    querystring,
                    parameters,
                    array_size,
                    self.pg_config.clone(),
                    None,
                )
            }
        }
    };
}

impl_cursor_method!(Transaction);
impl_cursor_method!(Connection);

macro_rules! impl_prepare_method {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            /// Create new prepared statement.
            ///
            /// # Errors
            /// May return error if there is some problem with DB communication.
            #[pyo3(signature = (querystring, parameters=None))]
            pub async fn prepare(
                &self,
                querystring: String,
                parameters: Option<pyo3::Py<PyAny>>,
            ) -> PSQLPyResult<PreparedStatement> {
                let Some(conn) = &self.conn else {
                    return Err(RustPSQLDriverError::ConnectionClosedError);
                };

                let read_conn_g = conn.read().await;
                let prep_stmt = read_conn_g
                    .prepare_statement(querystring, parameters)
                    .await?;

                Ok(PreparedStatement::new(
                    self.conn.clone(),
                    self.pg_config.clone(),
                    prep_stmt,
                ))
            }
        }
    };
}

impl_prepare_method!(Transaction);
impl_prepare_method!(Connection);

macro_rules! impl_transaction_methods {
    ($name:ident, $val:expr $(,)?) => {
        #[pymethods]
        impl $name {
            /// Commit existing transaction.
            ///
            /// # Errors
            /// May return error if there is some problem with DB communication.
            pub async fn commit(&mut self) -> PSQLPyResult<()> {
                let conn = self.conn.clone();
                let Some(conn) = conn else {
                    return Err(RustPSQLDriverError::TransactionClosedError);
                };
                let mut write_conn_g = conn.write().await;
                write_conn_g.commit().await?;

                if $val {
                    self.conn = None;
                }

                Ok(())
            }

            /// Rollback existing transaction.
            ///
            /// # Errors
            /// May return error if there is some problem with DB communication.
            pub async fn rollback(&mut self) -> PSQLPyResult<()> {
                let conn = self.conn.clone();
                let Some(conn) = conn else {
                    return Err(RustPSQLDriverError::TransactionClosedError);
                };
                let mut write_conn_g = conn.write().await;
                write_conn_g.rollback().await?;

                if $val {
                    self.conn = None;
                }

                Ok(())
            }
        }
    };
}

impl_transaction_methods!(Transaction, true);

macro_rules! impl_binary_copy_method {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            /// Perform binary copy to table.
            ///
            /// # Errors
            /// May return error if there is some problem with DB communication.
            #[pyo3(signature = (source, table_name, columns=None, schema_name=None))]
            pub async fn binary_copy_to_table(
                self_: pyo3::Py<Self>,
                source: Py<PyAny>,
                table_name: String,
                columns: Option<Vec<String>>,
                schema_name: Option<String>,
            ) -> PSQLPyResult<u64> {
                let (db_client, mut bytes_mut) =
                    Python::with_gil(|gil| -> PSQLPyResult<(Option<_>, BytesMut)> {
                        let db_client = self_.borrow(gil).conn.clone();

                        let Some(db_client) = db_client else {
                            return Ok((None, BytesMut::new()));
                        };

                        let data_bytes_mut =
                            if let Ok(py_buffer) = source.extract::<PyBuffer<u8>>(gil) {
                                let buffer_len = py_buffer.len_bytes();
                                let mut bytes_mut = BytesMut::zeroed(buffer_len);

                                py_buffer.copy_to_slice(gil, &mut bytes_mut[..])?;
                                bytes_mut
                            } else if let Ok(py_bytes) = source.call_method0(gil, "getvalue") {
                                if let Ok(bytes_vec) = py_bytes.extract::<Vec<u8>>(gil) {
                                    let bytes_mut = BytesMut::from(&bytes_vec[..]);
                                    bytes_mut
                                } else {
                                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                                        "source must be bytes or support Buffer protocol".into(),
                                    ));
                                }
                            } else {
                                return Err(RustPSQLDriverError::PyToRustValueConversionError(
                                    "source must be bytes or support Buffer protocol".into(),
                                ));
                            };

                        Ok((Some(db_client), data_bytes_mut))
                    })?;

                let Some(db_client) = db_client else {
                    return Ok(0);
                };

                let full_table_name = match schema_name {
                    Some(schema) => {
                        format!("{}.{}", quote_ident(&schema), quote_ident(&table_name))
                    }
                    None => quote_ident(&table_name),
                };

                let copy_qs = match columns {
                    Some(ref cols) if !cols.is_empty() => {
                        format!(
                            "COPY {}({}) FROM STDIN (FORMAT binary)",
                            full_table_name,
                            cols.join(", ")
                        )
                    }
                    _ => format!("COPY {} FROM STDIN (FORMAT binary)", full_table_name),
                };

                let read_conn_g = db_client.read().await;
                let sink = read_conn_g.copy_in(&copy_qs).await?;
                let writer = BinaryCopyInWriter::new_empty_buffer(sink, &[]);
                pin_mut!(writer);

                writer.as_mut().write_raw_bytes(&mut bytes_mut).await?;
                let rows_created = writer.as_mut().finish_empty().await?;

                Ok(rows_created)
            }
        }
    };
}

impl_binary_copy_method!(Connection);
impl_binary_copy_method!(Transaction);
