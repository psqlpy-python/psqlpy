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
use pyo3::{buffer::PyBuffer, PyErr, Python};
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
                let mut hosts_vec = vec![];

                let hosts = self.pg_config.get_hosts();
                for host in hosts {
                    match host {
                        Host::Tcp(host) => {
                            hosts_vec.push(host.to_string());
                        }
                        _ => unreachable!(),
                    }
                }

                hosts_vec
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
            pub fn cursor(
                &self,
                querystring: Option<String>,
                parameters: Option<Py<PyAny>>,
                array_size: Option<i32>,
            ) -> PSQLPyResult<Cursor> {
                Ok(Cursor::new(
                    self.conn.clone(),
                    querystring,
                    parameters,
                    array_size,
                    self.pg_config.clone(),
                    None,
                ))
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
            pub async fn commit(&mut self) -> PSQLPyResult<()> {
                let conn = self.conn.clone();
                let Some(conn) = conn else {
                    return Err(RustPSQLDriverError::TransactionClosedError("1".into()));
                };
                let mut write_conn_g = conn.write().await;
                write_conn_g.commit().await?;

                if $val {
                    self.conn = None;
                }

                Ok(())
            }

            pub async fn rollback(&mut self) -> PSQLPyResult<()> {
                let conn = self.conn.clone();
                let Some(conn) = conn else {
                    return Err(RustPSQLDriverError::TransactionClosedError("2".into()));
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
            #[pyo3(signature = (source, table_name, columns=None, schema_name=None))]
            pub async fn binary_copy_to_table(
                self_: pyo3::Py<Self>,
                source: Py<PyAny>,
                table_name: String,
                columns: Option<Vec<String>>,
                schema_name: Option<String>,
            ) -> PSQLPyResult<u64> {
                let db_client = pyo3::Python::with_gil(|gil| self_.borrow(gil).conn.clone());
                let mut table_name = quote_ident(&table_name);
                if let Some(schema_name) = schema_name {
                    table_name = format!("{}.{}", quote_ident(&schema_name), table_name);
                }

                let mut formated_columns = String::default();
                if let Some(columns) = columns {
                    formated_columns = format!("({})", columns.join(", "));
                }

                let copy_qs =
                    format!("COPY {table_name}{formated_columns} FROM STDIN (FORMAT binary)");

                if let Some(db_client) = db_client {
                    let mut psql_bytes: BytesMut = Python::with_gil(|gil| {
                        let possible_py_buffer: Result<PyBuffer<u8>, PyErr> =
                            source.extract::<PyBuffer<u8>>(gil);
                        if let Ok(py_buffer) = possible_py_buffer {
                            let vec_buf = py_buffer.to_vec(gil)?;
                            return Ok(BytesMut::from(vec_buf.as_slice()));
                        }

                        if let Ok(py_bytes) = source.call_method0(gil, "getvalue") {
                            if let Ok(bytes) = py_bytes.extract::<Vec<u8>>(gil) {
                                return Ok(BytesMut::from(bytes.as_slice()));
                            }
                        }

                        Err(RustPSQLDriverError::PyToRustValueConversionError(
                            "source must be bytes or support Buffer protocol".into(),
                        ))
                    })?;

                    let read_conn_g = db_client.read().await;
                    let sink = read_conn_g.copy_in(&copy_qs).await?;
                    let writer = BinaryCopyInWriter::new_empty_buffer(sink, &[]);
                    pin_mut!(writer);
                    writer.as_mut().write_raw_bytes(&mut psql_bytes).await?;
                    let rows_created = writer.as_mut().finish_empty().await?;
                    return Ok(rows_created);
                }

                Ok(0)
            }
        }
    };
}

impl_binary_copy_method!(Connection);
impl_binary_copy_method!(Transaction);
