use pyo3::{
    prelude::*,
    pyclass, pymethods,
    types::{PyDict, PyTuple},
    IntoPyObjectExt, Py, PyAny, Python,
};
use tokio_postgres::Row;

use crate::{exceptions::rust_errors::PSQLPyResult, value_converter::to_python::postgres_to_py};

/// Convert postgres `Row` into Python Dict.
///
/// # Errors
///
/// May return Err Result if can not convert
/// postgres type to python or set new key-value pair
/// in python dict.
#[allow(clippy::ref_option)]
fn row_to_dict<'a>(
    py: Python<'a>,
    postgres_row: &'a Row,
    custom_decoders: &Option<Py<PyDict>>,
) -> PSQLPyResult<Bound<'a, PyDict>> {
    let python_dict = PyDict::new(py);
    for (column_idx, column) in postgres_row.columns().iter().enumerate() {
        let python_type = postgres_to_py(py, postgres_row, column, column_idx, custom_decoders)?;
        python_dict.set_item(column.name().into_py_any(py)?, python_type)?;
    }
    Ok(python_dict)
}

/// Convert postgres `Row` into Python Tuple.
///
/// # Errors
///
/// May return Err Result if can not convert
/// postgres type to python or set new key-value pair
/// in python dict.
#[allow(clippy::ref_option)]
fn row_to_tuple<'a>(
    py: Python<'a>,
    postgres_row: &'a Row,
    custom_decoders: &Option<Py<PyDict>>,
) -> PSQLPyResult<Bound<'a, PyTuple>> {
    let columns = postgres_row.columns();
    let mut tuple_items = Vec::with_capacity(columns.len());

    for (column_idx, column) in columns.iter().enumerate() {
        let python_value = postgres_to_py(py, postgres_row, column, column_idx, custom_decoders)?;
        tuple_items.push(python_value);
    }

    Ok(PyTuple::new(py, tuple_items)?)
}

#[pyclass(name = "QueryResult")]
#[allow(clippy::module_name_repetitions)]
pub struct PSQLDriverPyQueryResult {
    pub inner: Vec<Row>,
}

impl PSQLDriverPyQueryResult {
    #[must_use]
    pub fn new(database_result: Vec<Row>) -> Self {
        PSQLDriverPyQueryResult {
            inner: database_result,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
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
    #[pyo3(signature = (custom_decoders=None, as_tuple=None))]
    #[allow(clippy::needless_pass_by_value)]
    pub fn result(
        &self,
        py: Python<'_>,
        custom_decoders: Option<Py<PyDict>>,
        as_tuple: Option<bool>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let as_tuple = as_tuple.unwrap_or(false);

        if as_tuple {
            let mut tuple_rows: Vec<Bound<'_, PyTuple>> = vec![];
            for row in &self.inner {
                tuple_rows.push(row_to_tuple(py, row, &custom_decoders)?);
            }
            return Ok(tuple_rows.into_py_any(py)?);
        }

        let mut dict_rows: Vec<Bound<'_, PyDict>> = vec![];
        for row in &self.inner {
            dict_rows.push(row_to_dict(py, row, &custom_decoders)?);
        }
        Ok(dict_rows.into_py_any(py)?)
    }

    /// Convert result from database to any class passed from Python.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type to python or create new Python class.
    #[allow(clippy::needless_pass_by_value)]
    pub fn as_class<'a>(&'a self, py: Python<'a>, as_class: Py<PyAny>) -> PSQLPyResult<Py<PyAny>> {
        let mut result: Vec<Py<PyAny>> = vec![];
        for row in &self.inner {
            let pydict: pyo3::Bound<'_, PyDict> = row_to_dict(py, row, &None)?;
            let convert_class_inst = as_class.call(py, (), Some(&pydict))?;
            result.push(convert_class_inst);
        }

        Ok(result.into_py_any(py)?)
    }

    /// Convert result from database with function passed from Python.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type with custom function.
    #[allow(clippy::needless_pass_by_value)]
    #[pyo3(signature = (row_factory, custom_decoders=None))]
    pub fn row_factory<'a>(
        &'a self,
        py: Python<'a>,
        row_factory: Py<PyAny>,
        custom_decoders: Option<Py<PyDict>>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let mut result: Vec<Py<PyAny>> = vec![];
        for row in &self.inner {
            let pydict: pyo3::Bound<'_, PyDict> = row_to_dict(py, row, &custom_decoders)?;
            let row_factory_class = row_factory.call(py, (pydict,), None)?;
            result.push(row_factory_class);
        }
        Ok(result.into_py_any(py)?)
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
    #[allow(clippy::needless_pass_by_value)]
    #[pyo3(signature = (custom_decoders=None, as_tuple=None))]
    pub fn result(
        &self,
        py: Python<'_>,
        custom_decoders: Option<Py<PyDict>>,
        as_tuple: Option<bool>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let as_tuple = as_tuple.unwrap_or(false);

        if as_tuple {
            return Ok(row_to_tuple(py, &self.inner, &custom_decoders)?.into_py_any(py)?);
        }

        Ok(row_to_dict(py, &self.inner, &custom_decoders)?.into_py_any(py)?)
    }

    /// Convert result from database to any class passed from Python.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type to python, can not create new Python class
    /// or there are no results.
    #[allow(clippy::needless_pass_by_value)]
    pub fn as_class<'a>(&'a self, py: Python<'a>, as_class: Py<PyAny>) -> PSQLPyResult<Py<PyAny>> {
        let pydict: pyo3::Bound<'_, PyDict> = row_to_dict(py, &self.inner, &None)?;
        Ok(as_class.call(py, (), Some(&pydict))?)
    }

    /// Convert result from database with function passed from Python.
    ///
    /// # Errors
    ///
    /// May return Err Result if can not convert
    /// postgres type with custom function
    #[allow(clippy::needless_pass_by_value)]
    #[pyo3(signature = (row_factory, custom_decoders=None))]
    pub fn row_factory<'a>(
        &'a self,
        py: Python<'a>,
        row_factory: Py<PyAny>,
        custom_decoders: Option<Py<PyDict>>,
    ) -> PSQLPyResult<Py<PyAny>> {
        let pydict = row_to_dict(py, &self.inner, &custom_decoders)?.into_py_any(py)?;
        Ok(row_factory.call(py, (pydict,), None)?)
    }
}
