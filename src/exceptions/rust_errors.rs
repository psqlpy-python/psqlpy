use thiserror::Error;

use super::python_errors::RustPSQLDriverPyBaseError;

pub type RustPSQLDriverPyResult<T> = Result<T, RustPSQLDriverError>;

#[derive(Error, Debug)]
pub enum RustPSQLDriverError {
    #[error("Database pool error: {0}.")]
    DatabasePoolError(String),
    #[error("Can't convert value from engine to python type: {0}")]
    RustToPyValueConversionError(String),
    #[error("Can't convert value from python to rust type: {0}")]
    PyToRustValueConversionError(String),
    #[error("Transaction exception: {0}")]
    DBTransactionError(String),
    #[error("Configuration database pool error: {0}")]
    DBPoolConfigurationError(String),

    #[error("Python exception: {0}.")]
    PyError(#[from] pyo3::PyErr),
    #[error("Database engine exception: {0}.")]
    DBEngineError(#[from] tokio_postgres::Error),
    #[error("Database engine pool exception: {0}")]
    DBEnginePoolError(#[from] deadpool_postgres::PoolError),
    #[error("Database engine build failed: {0}")]
    DBEngineBuildError(#[from] deadpool_postgres::BuildError),
}

impl From<RustPSQLDriverError> for pyo3::PyErr {
    fn from(error: RustPSQLDriverError) -> Self {
        let error_desc = error.to_string();
        match error {
            RustPSQLDriverError::PyError(err) => err,
            RustPSQLDriverError::DBEngineError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::RustToPyValueConversionError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::PyToRustValueConversionError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::DatabasePoolError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::DBEnginePoolError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::DBEngineBuildError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::DBTransactionError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
            RustPSQLDriverError::DBPoolConfigurationError(_) => {
                RustPSQLDriverPyBaseError::new_err((error_desc,))
            }
        }
    }
}
