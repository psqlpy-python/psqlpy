use pyo3::{pyclass, pymethods, types::PyDict, Py, PyAny, Python, ToPyObject};
use tokio_postgres::Row;

use crate::{exceptions::rust_errors::RustPSQLDriverPyResult, value_converter::postgres_to_py};

/// Convert postgres `Row` into Python Dict.
///
/// # Errors
///
/// May return Err Result if can not convert
/// postgres type to python or set new key-value pair
/// in python dict.
fn row_to_dict<'a>(py: Python<'a>, postgres_row: &'a Row) -> RustPSQLDriverPyResult<&'a PyDict> {
    let python_dict = PyDict::new(py);
    for (column_idx, column) in postgres_row.columns().iter().enumerate() {
        let python_type = postgres_to_py(py, postgres_row, column, column_idx)?;
        python_dict.set_item(column.name().to_object(py), python_type)?;
    }
    Ok(python_dict)
}

#[pyclass(name = "QueryResult")]
#[allow(clippy::module_name_repetitions)]
pub struct PSQLDriverPyQueryResult {
    inner: Vec<Row>,
}

impl PSQLDriverPyQueryResult {
    #[must_use]
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
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type to python or set new key-value pair
    /// in python dict.
    pub fn result(&self, py: Python<'_>) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let mut result: Vec<&PyDict> = vec![];
        for row in &self.inner {
            result.push(row_to_dict(py, row)?);
        }
        Ok(result.to_object(py))
    }

    /// Convert result from database to any class passed from Python.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type to python or create new Python class.
    pub fn as_class<'a>(
        &'a self,
        py: Python<'a>,
        as_class: &'a PyAny,
    ) -> RustPSQLDriverPyResult<Py<PyAny>> {
        let mut res: Vec<&PyAny> = vec![];
        for row in &self.inner {
            let pydict: &PyDict = row_to_dict(py, row)?;
            let convert_class_inst = as_class.call((), Some(pydict))?;
            res.push(convert_class_inst);
        }

        Ok(res.to_object(py))
    }
}

#[pyclass(name = "SingleQueryResult")]
#[allow(clippy::module_name_repetitions)]
pub struct PSQLDriverSinglePyQueryResult {
    inner: Row,
}

impl PSQLDriverSinglePyQueryResult {
    #[must_use]
    pub fn new(database_row: Row) -> Self {
        PSQLDriverSinglePyQueryResult {
            inner: database_row,
        }
    }

    pub fn get_inner(self) -> Row {
        self.inner
    }
}

#[pymethods]
impl PSQLDriverSinglePyQueryResult {
    /// Return result as a Python dict.
    ///
    /// This result is used to return single row.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type to python, can not set new key-value pair
    /// in python dict or there are no result.
    pub fn result(&self, py: Python<'_>) -> RustPSQLDriverPyResult<Py<PyAny>> {
        Ok(row_to_dict(py, &self.inner)?.to_object(py))
    }

    /// Convert result from database to any class passed from Python.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type to python, can not create new Python class
    /// or there are no results.
    pub fn as_class<'a>(
        &'a self,
        py: Python<'a>,
        as_class: &'a PyAny,
    ) -> RustPSQLDriverPyResult<&'a PyAny> {
        let pydict: &PyDict = row_to_dict(py, &self.inner)?;
        Ok(as_class.call((), Some(pydict))?)
    }
}
