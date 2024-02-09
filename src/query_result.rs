use pyo3::{pyclass, pymethods, types::PyDict, Py, PyAny, Python, ToPyObject};
use tokio_postgres::Row;

use crate::{exceptions::rust_errors::RustPSQLDriverPyResult, value_converter::postgres_to_py};

#[pyclass(name = "QueryResult")]
pub struct PSQLDriverPyQueryResult {
    inner: Vec<Row>,
}

impl PSQLDriverPyQueryResult {
    pub fn new(database_result: Vec<Row>) -> Self {
        PSQLDriverPyQueryResult {
            inner: database_result,
        }
    }
}

#[pymethods]
impl PSQLDriverPyQueryResult {
    /// Return result as a Python list of dicts.
    ///
    /// It's a common variant how to return a result for the future
    /// processing.
    ///
    /// # Errors:
    ///
    /// May return Err Result if can not convert
    /// postgres type to python or set new key-value pair
    /// in python dict.
    pub fn result<'a>(&self, py: Python<'a>) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let mut result: Vec<&PyDict> = vec![];
        for row in &self.inner {
            let python_dict = PyDict::new(py);
            for (column_idx, column) in row.columns().iter().enumerate() {
                let python_type = postgres_to_py(py, row, column, column_idx)?;
                python_dict.set_item(column.name().to_object(py), python_type)?;
            }
            result.push(python_dict);
        }
        Ok(result.to_object(py))
    }
}
