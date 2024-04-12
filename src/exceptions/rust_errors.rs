use thiserror::Error;
use tokio::task::JoinError;

use crate::exceptions::python_errors::{
    DBPoolConfigurationError, DBPoolError, PyToRustValueMappingError, RustPSQLDriverPyBaseError,
    RustToPyValueMappingError, TransactionError,
};

use super::python_errors::{CursorError, UUIDValueConvertError};

pub type RustPSQLDriverPyResult<T> = Result<T, RustPSQLDriverError>;

#[derive(Error, Debug)]
pub enum RustPSQLDriverError {
    #[error("Database pool error: {0}.")]
    DatabasePoolError(String),
    #[error("Can't convert value from driver to python type: {0}")]
    RustToPyValueConversionError(String),
    #[error("Can't convert value from python to rust type: {0}")]
    PyToRustValueConversionError(String),
    #[error("Transaction exception: {0}")]
    DataBaseTransactionError(String),
    #[error("Configuration database pool error: {0}")]
    DataBasePoolConfigurationError(String),
    #[error("Cursor error: {0}")]
    DataBaseCursorError(String),

    #[error("Python exception: {0}.")]
    PyError(#[from] pyo3::PyErr),
    #[error("Database engine exception: {0}.")]
    DBEngineError(#[from] deadpool_postgres::tokio_postgres::Error),
    #[error("Database engine pool exception: {0}")]
    DBEnginePoolError(#[from] deadpool_postgres::PoolError),
    #[error("Database engine build failed: {0}")]
    DBEngineBuildError(#[from] deadpool_postgres::BuildError),
    #[error("Value convert has failed: {0}")]
    UUIDConvertError(#[from] uuid::Error),
    #[error("Cannot convert provided string to MacAddr6")]
    MacAddr6ConversionError(#[from] macaddr::ParseError),
    #[error("Cannot execute future in Rust: {0}")]
    RuntimeJoinError(#[from] JoinError),
}

impl From<RustPSQLDriverError> for pyo3::PyErr {
    fn from(error: RustPSQLDriverError) -> Self {
        let error_desc = error.to_string();
        match error {
            RustPSQLDriverError::PyError(err) => err,
            RustPSQLDriverError::DBEngineError(_)
            | RustPSQLDriverError::DBEnginePoolError(_)
            | RustPSQLDriverError::MacAddr6ConversionError(_)
            | RustPSQLDriverError::DBEngineBuildError(_)
            | RustPSQLDriverError::RuntimeJoinError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::RustToPyValueConversionError(_) => {
                RustToPyValueMappingError::new_err((error_desc,))
            }
            RustPSQLDriverError::PyToRustValueConversionError(_) => {
                PyToRustValueMappingError::new_err((error_desc,))
            }
            RustPSQLDriverError::DatabasePoolError(_) => DBPoolError::new_err((error_desc,)),
            RustPSQLDriverError::DataBaseTransactionError(_) => {
                TransactionError::new_err((error_desc,))
            }
            RustPSQLDriverError::DataBasePoolConfigurationError(_) => {
                DBPoolConfigurationError::new_err((error_desc,))
            }
            RustPSQLDriverError::UUIDConvertError(_) => UUIDValueConvertError::new_err(error_desc),
            RustPSQLDriverError::DataBaseCursorError(_) => CursorError::new_err(error_desc),
        }
    }
}
