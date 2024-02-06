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
