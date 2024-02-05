use pyo3::{pyclass, pymethods, types::PyDict, Py, PyAny, Python, ToPyObject};
use tokio_postgres::Row;

use crate::{exceptions::rust_errors::RustEnginePyResult, value_converter::postgres_to_py};

#[pyclass(name = "QueryResult")]
pub struct RustEnginePyQueryResult {
    inner: Vec<Row>,
}

impl RustEnginePyQueryResult {
    pub fn new(database_result: Vec<Row>) -> Self {
        RustEnginePyQueryResult {
            inner: database_result,
        }
    }
}

#[pymethods]
impl RustEnginePyQueryResult {
    pub fn result<'a>(&self, py: Python<'a>) -> RustEnginePyResult<Py<PyAny>> {
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
