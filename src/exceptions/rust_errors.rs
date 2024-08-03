use openssl::error::ErrorStack;
use thiserror::Error;
use tokio::task::JoinError;

use crate::exceptions::python_errors::{PyToRustValueMappingError, RustToPyValueMappingError};

use super::python_errors::{
    BaseConnectionError, BaseConnectionPoolError, BaseCursorError, BaseTransactionError,
    ConnectionClosedError, ConnectionExecuteError, ConnectionPoolBuildError,
    ConnectionPoolConfigurationError, ConnectionPoolExecuteError, CursorCloseError,
    CursorClosedError, CursorFetchError, CursorStartError, DriverError, MacAddrParseError,
    RuntimeJoinError, SSLError, TransactionBeginError, TransactionClosedError,
    TransactionCommitError, TransactionExecuteError, TransactionRollbackError,
    TransactionSavepointError, UUIDValueConvertError,
};

pub type RustPSQLDriverPyResult<T> = Result<T, RustPSQLDriverError>;

#[derive(Error, Debug)]
pub enum RustPSQLDriverError {
    // ConnectionPool errors
    #[error("Connection pool error: {0}.")]
    BaseConnectionPoolError(String),
    #[error("Connection pool build error: {0}.")]
    ConnectionPoolBuildError(String),
    #[error("Connection pool configuration error: {0}.")]
    ConnectionPoolConfigurationError(String),
    #[error("Connection pool execute error: {0}.")]
    ConnectionPoolExecuteError(String),

    // Connection Errors
    #[error("Connection error: {0}.")]
    BaseConnectionError(String),
    #[error("Connection execute error: {0}.")]
    ConnectionExecuteError(String),
    #[error("Underlying connection is returned to the pool")]
    ConnectionClosedError,

    // Transaction Errors
    #[error("Transaction error: {0}")]
    BaseTransactionError(String),
    #[error("Transaction begin error: {0}")]
    TransactionBeginError(String),
    #[error("Transaction commit error: {0}")]
    TransactionCommitError(String),
    #[error("Transaction rollback error: {0}")]
    TransactionRollbackError(String),
    #[error("Transaction savepoint error: {0}")]
    TransactionSavepointError(String),
    #[error("Transaction execute error: {0}")]
    TransactionExecuteError(String),
    #[error("Underlying connection is returned to the pool")]
    TransactionClosedError,

    // Cursor Errors
    #[error("Cursor error: {0}")]
    BaseCursorError(String),
    #[error("Cursor start error: {0}")]
    CursorStartError(String),
    #[error("Cursor close error: {0}")]
    CursorCloseError(String),
    #[error("Cursor fetch error: {0}")]
    CursorFetchError(String),
    #[error("Underlying connection is returned to the pool")]
    CursorClosedError,

    #[error("Can't convert value from driver to python type: {0}")]
    RustToPyValueConversionError(String),
    #[error("Can't convert value from python to rust type: {0}")]
    PyToRustValueConversionError(String),

    #[error("Python exception: {0}.")]
    RustPyError(#[from] pyo3::PyErr),
    #[error("Database engine exception: {0}.")]
    RustDriverError(#[from] deadpool_postgres::tokio_postgres::Error),
    #[error("Database engine pool exception: {0}")]
    RustConnectionPoolError(#[from] deadpool_postgres::PoolError),
    #[error("Database engine build failed: {0}")]
    RustDriverBuildError(#[from] deadpool_postgres::BuildError),
    #[error("Value convert has failed: {0}")]
    RustUUIDConvertError(#[from] uuid::Error),
    #[error("Cannot convert provided string to MacAddr6")]
    RustMacAddrConversionError(#[from] macaddr::ParseError),
    #[error("Cannot execute future in Rust: {0}")]
    RustRuntimeJoinError(#[from] JoinError),
    #[error("Cannot convert python Decimal into rust Decimal")]
    DecimalConversionError(#[from] rust_decimal::Error),
    #[error("Cannot create set SSL: {0}")]
    SSLError(#[from] ErrorStack),
}

impl From<RustPSQLDriverError> for pyo3::PyErr {
    fn from(error: RustPSQLDriverError) -> Self {
        let error_desc = error.to_string();
        match error {
            RustPSQLDriverError::RustPyError(err) => err,
            RustPSQLDriverError::RustDriverError(_) => DriverError::new_err((error_desc,)),
            RustPSQLDriverError::RustMacAddrConversionError(_) => {
                MacAddrParseError::new_err((error_desc,))
            }
            RustPSQLDriverError::RustRuntimeJoinError(_) => {
                RuntimeJoinError::new_err((error_desc,))
            }
            RustPSQLDriverError::RustToPyValueConversionError(_) => {
                RustToPyValueMappingError::new_err((error_desc,))
            }
            RustPSQLDriverError::PyToRustValueConversionError(_)
            | RustPSQLDriverError::DecimalConversionError(_) => {
                PyToRustValueMappingError::new_err((error_desc,))
            }
            RustPSQLDriverError::ConnectionPoolConfigurationError(_) => {
                ConnectionPoolConfigurationError::new_err((error_desc,))
            }
            RustPSQLDriverError::RustUUIDConvertError(_) => {
                UUIDValueConvertError::new_err(error_desc)
            }
            RustPSQLDriverError::BaseConnectionPoolError(_)
            | RustPSQLDriverError::RustConnectionPoolError(_) => {
                BaseConnectionPoolError::new_err((error_desc,))
            }
            RustPSQLDriverError::ConnectionPoolBuildError(_)
            | RustPSQLDriverError::RustDriverBuildError(_) => {
                ConnectionPoolBuildError::new_err((error_desc,))
            }
            RustPSQLDriverError::ConnectionPoolExecuteError(_) => {
                ConnectionPoolExecuteError::new_err((error_desc,))
            }
            RustPSQLDriverError::BaseConnectionError(_) => {
                BaseConnectionError::new_err((error_desc,))
            }
            RustPSQLDriverError::ConnectionExecuteError(_) => {
                ConnectionExecuteError::new_err((error_desc,))
            }
            RustPSQLDriverError::ConnectionClosedError => {
                ConnectionClosedError::new_err((error_desc,))
            }
            RustPSQLDriverError::BaseTransactionError(_) => {
                BaseTransactionError::new_err((error_desc,))
            }
            RustPSQLDriverError::TransactionBeginError(_) => {
                TransactionBeginError::new_err((error_desc,))
            }
            RustPSQLDriverError::TransactionCommitError(_) => {
                TransactionCommitError::new_err((error_desc,))
            }
            RustPSQLDriverError::TransactionRollbackError(_) => {
                TransactionRollbackError::new_err((error_desc,))
            }
            RustPSQLDriverError::TransactionSavepointError(_) => {
                TransactionSavepointError::new_err((error_desc,))
            }
            RustPSQLDriverError::TransactionExecuteError(_) => {
                TransactionExecuteError::new_err((error_desc,))
            }
            RustPSQLDriverError::TransactionClosedError => {
                TransactionClosedError::new_err((error_desc,))
            }
            RustPSQLDriverError::BaseCursorError(_) => BaseCursorError::new_err((error_desc,)),
            RustPSQLDriverError::CursorStartError(_) => CursorStartError::new_err((error_desc,)),
            RustPSQLDriverError::CursorCloseError(_) => CursorCloseError::new_err((error_desc,)),
            RustPSQLDriverError::CursorFetchError(_) => CursorFetchError::new_err((error_desc,)),
            RustPSQLDriverError::SSLError(_) => SSLError::new_err((error_desc,)),
            RustPSQLDriverError::CursorClosedError => CursorClosedError::new_err((error_desc,)),
        }
    }
}
