use thiserror::Error;

use super::python_errors::RustEnginePyBaseError;

pub type RustEnginePyResult<T> = Result<T, RustEngineError>;

#[derive(Error, Debug)]
pub enum RustEngineError {
    #[error("Database pool error: {0}.")]
    DatabasePoolError(String),
    #[error("Can't convert value from engine to python type: {0}")]
    RustToPyValueConversionError(String),
    #[error("Can't convert value from python to rust type: {0}")]
    PyToRustValueConversionError(String),
    #[error("Transaction exception: {0}")]
    DBTransactionError(String),

    #[error("Python exception: {0}.")]
    PyError(#[from] pyo3::PyErr),
    #[error("Database engine exception: {0}.")]
    DBEngineError(#[from] tokio_postgres::Error),
    #[error("Database engine pool exception: {0}")]
    DBEnginePoolError(#[from] deadpool_postgres::PoolError),
    #[error("Database engine build failed: {0}")]
    DBEngineBuildError(#[from] deadpool_postgres::BuildError),
}

impl From<RustEngineError> for pyo3::PyErr {
    fn from(error: RustEngineError) -> Self {
        let error_desc = error.to_string();
        match error {
            RustEngineError::PyError(err) => err,
            RustEngineError::DBEngineError(_) => RustEnginePyBaseError::new_err((error_desc,)),
            RustEngineError::RustToPyValueConversionError(_) => {
                RustEnginePyBaseError::new_err((error_desc,))
            }
            RustEngineError::PyToRustValueConversionError(_) => {
                RustEnginePyBaseError::new_err((error_desc,))
            }
            RustEngineError::DatabasePoolError(_) => RustEnginePyBaseError::new_err((error_desc,)),
            RustEngineError::DBEnginePoolError(_) => RustEnginePyBaseError::new_err((error_desc,)),
            RustEngineError::DBEngineBuildError(_) => RustEnginePyBaseError::new_err((error_desc,)),
            RustEngineError::DBTransactionError(_) => RustEnginePyBaseError::new_err((error_desc,)),
        }
    }
}
