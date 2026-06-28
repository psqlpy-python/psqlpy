use tokio_postgres::config::Host;

use std::net::IpAddr;

use super::{
    connection::Connection, cursor::Cursor, prepared_statement::PreparedStatement,
    transaction::Transaction,
};

use pyo3::{pymethods, Py, PyAny};

use crate::{
    connection::traits::{CloseTransaction, Connection as _},
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    value_converter::{dto::enums::PythonDTO, from_python::from_python_typed},
};

use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use futures_util::pin_mut;
use pyo3::{buffer::PyBuffer, types::PyAnyMethods, Python};
use tokio_postgres::{
    binary_copy::BinaryCopyInWriter,
    types::{IsNull, ToSql},
};

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

/// Asyncpg's `_COPY_BUFFER_SIZE`: flush when the encode buffer reaches 512 KiB.
const COPY_BUFFER_SIZE: usize = 524_288;

/// `PostgreSQL` binary COPY file header.
const COPY_MAGIC: &[u8] = b"PGCOPY\n\xff\r\n\0";

/// Encode one field into `buf` using the `PostgreSQL` binary COPY wire format:
/// a 4-byte big-endian length prefix followed by the serialised value,
/// or -1 (as i32) for NULL.
pub(crate) fn encode_copy_field(
    buf: &mut BytesMut,
    dto: &PythonDTO,
    ty: &tokio_postgres::types::Type,
) -> PSQLPyResult<()> {
    let len_pos = buf.len();
    buf.put_i32(0); // placeholder — overwritten after encoding
    let data_start = buf.len();
    let is_null = dto.to_sql_checked(ty, buf).map_err(|e| {
        RustPSQLDriverError::PyToRustValueConversionError(format!("COPY binary encode error: {e}"))
    })?;
    // COPY field lengths fit in i32; a single encoded field cannot exceed 2 GiB.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let field_len = match is_null {
        IsNull::No => (buf.len() - data_start) as i32,
        IsNull::Yes => {
            // NULL: truncate back to placeholder position, no data bytes
            buf.truncate(data_start);
            -1i32
        }
    };
    BigEndian::write_i32(&mut buf[len_pos..], field_len);
    Ok(())
}

macro_rules! impl_copy_records_method {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            /// Copy a list of records into a table using the COPY FROM STDIN
            /// binary protocol.
            ///
            /// Column types are introspected from the target table, so callers
            /// pass Python values directly (the same conversions used by
            /// `execute`). Mirrors `asyncpg.Connection.copy_records_to_table`.
            ///
            /// The encoder follows asyncpg's algorithm: a single `BytesMut`
            /// accumulator flushed into 512 KiB (`_COPY_BUFFER_SIZE`) chunks.
            /// All rows are encoded during the GIL pass; chunks are sent to the
            /// server in a second pass after the GIL is released.
            ///
            /// # Errors
            /// May return error if there is some problem with DB communication,
            /// the table cannot be introspected, or a value cannot be converted.
            #[allow(clippy::too_many_lines)]
            #[pyo3(signature = (table_name, records, columns=None, schema_name=None))]
            pub async fn copy_records_to_table(
                self_: pyo3::Py<Self>,
                table_name: String,
                records: Py<PyAny>,
                columns: Option<Vec<String>>,
                schema_name: Option<String>,
            ) -> PSQLPyResult<u64> {
                let db_client = Python::with_gil(|gil| self_.borrow(gil).conn.clone());

                let Some(db_client) = db_client else {
                    return Ok(0);
                };

                let full_table_name = match schema_name.as_deref() {
                    Some(schema) => {
                        format!("{}.{}", quote_ident(schema), quote_ident(&table_name))
                    }
                    None => quote_ident(&table_name),
                };

                let columns_sql: Option<String> = match columns {
                    Some(ref cols) if !cols.is_empty() => Some(
                        cols.iter()
                            .map(|c| quote_ident(c))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    _ => None,
                };

                let read_conn_g = db_client.read().await;

                // Consult the per-connection type cache before issuing an
                // introspection query (avoids PREPARE+DEALLOCATE round-trips).
                let cache_key = (
                    schema_name.clone(),
                    table_name.clone(),
                    columns.clone().unwrap_or_default(),
                );
                let column_types: Vec<tokio_postgres::types::Type> = if let Some(cached) =
                    read_conn_g.copy_type_cache().get(&cache_key)
                {
                    (*cached).clone()
                } else {
                    let introspect_qs = match &columns_sql {
                        Some(cols) => {
                            format!("SELECT {} FROM {} WHERE false", cols, full_table_name)
                        }
                        None => format!("SELECT * FROM {} WHERE false", full_table_name),
                    };
                    let stmt = read_conn_g.prepare(&introspect_qs, false).await?;
                    let types: Vec<_> = stmt.columns().iter().map(|c| c.type_().clone()).collect();
                    read_conn_g
                        .copy_type_cache()
                        .insert(cache_key, types.clone());
                    types
                };

                if column_types.is_empty() {
                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                        "Cannot introspect column types from target table".into(),
                    ));
                }

                let copy_qs = match &columns_sql {
                    Some(cols) => format!(
                        "COPY {}({}) FROM STDIN (FORMAT binary)",
                        full_table_name, cols
                    ),
                    None => format!("COPY {} FROM STDIN (FORMAT binary)", full_table_name),
                };

                let sink = read_conn_g.copy_in::<_, bytes::Bytes>(&copy_qs).await?;

                // GIL pass: encode all rows into `COPY_BUFFER_SIZE` chunks.
                // After the GIL is released, the chunks are sent to the server.
                // This eliminates the prior two-phase approach (materialise
                // Vec<Vec<Py<PyAny>>> then re-visit for DTO conversion).
                let mut chunks: Vec<bytes::Bytes> = Vec::new();

                let gil_result: PSQLPyResult<()> = Python::with_gil(|gil| {
                    let n_cols = column_types.len();
                    let mut buf = BytesMut::with_capacity(COPY_BUFFER_SIZE);
                    // Scratch vec allocated once and cleared between rows (T3#10).
                    let mut cells_scratch: Vec<pyo3::Bound<'_, pyo3::PyAny>> =
                        Vec::with_capacity(n_cols);

                    // COPY binary file header
                    buf.put_slice(COPY_MAGIC);
                    buf.put_i32(0); // flags
                    buf.put_i32(0); // header extension length

                    for (row_idx, item) in records.bind(gil).try_iter()?.enumerate() {
                        let row = item?;
                        cells_scratch.clear();
                        for cell in row.try_iter()? {
                            cells_scratch.push(cell?);
                        }

                        if cells_scratch.len() != n_cols {
                            return Err(RustPSQLDriverError::PyToRustValueConversionError(
                                format!(
                                    "Record at index {} has {} fields, expected {}",
                                    row_idx,
                                    cells_scratch.len(),
                                    n_cols
                                ),
                            ));
                        }

                        // PostgreSQL max columns = 1600, well within i16 range.
                        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                        buf.put_i16(n_cols as i16);
                        for (cell, ty) in cells_scratch.iter().zip(column_types.iter()) {
                            let dto = from_python_typed(cell, ty)?;
                            encode_copy_field(&mut buf, &dto, ty)?;
                        }

                        if buf.len() >= COPY_BUFFER_SIZE {
                            chunks.push(buf.split().freeze());
                        }
                    }

                    // Binary COPY trailer
                    buf.put_i16(-1);
                    chunks.push(buf.freeze());
                    Ok(())
                });

                pin_mut!(sink);

                if let Err(e) = gil_result {
                    // Abort the sink so the server sees copy_fail + ReadyForQuery
                    // rather than a silent connection-level drop.
                    use futures_util::SinkExt;
                    let _ = sink.close().await;
                    return Err(e);
                }

                // Send all chunks outside the GIL.
                for chunk in chunks {
                    use futures_util::SinkExt;
                    sink.send(chunk).await?;
                }
                let rows_created = sink.finish().await?;

                Ok(rows_created)
            }
        }
    };
}

impl_copy_records_method!(Connection);
impl_copy_records_method!(Transaction);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_converter::dto::enums::PythonDTO;
    use byteorder::BigEndian;
    use bytes::BytesMut;
    use tokio_postgres::types::Type;

    fn decode_i32(buf: &[u8], offset: usize) -> i32 {
        BigEndian::read_i32(&buf[offset..offset + 4])
    }

    #[test]
    fn encode_copy_field_integer() {
        let dto = PythonDTO::PyIntI32(42i32);
        let mut buf = BytesMut::new();
        encode_copy_field(&mut buf, &dto, &Type::INT4).unwrap();
        // 4-byte length prefix + 4-byte INT4 payload
        assert_eq!(buf.len(), 8);
        assert_eq!(decode_i32(&buf, 0), 4); // field length
        assert_eq!(decode_i32(&buf, 4), 42); // value
    }

    #[test]
    fn encode_copy_field_null() {
        let dto = PythonDTO::PyNone;
        let mut buf = BytesMut::new();
        encode_copy_field(&mut buf, &dto, &Type::INT4).unwrap();
        // NULL: 4-byte length prefix = -1, no payload
        assert_eq!(buf.len(), 4);
        assert_eq!(decode_i32(&buf, 0), -1);
    }

    #[test]
    fn encode_copy_field_text() {
        let dto = PythonDTO::PyText("hi".to_string());
        let mut buf = BytesMut::new();
        encode_copy_field(&mut buf, &dto, &Type::TEXT).unwrap();
        // 4-byte length prefix + 2 bytes of UTF-8
        assert_eq!(buf.len(), 6);
        assert_eq!(decode_i32(&buf, 0), 2);
        assert_eq!(&buf[4..], b"hi");
    }
}
