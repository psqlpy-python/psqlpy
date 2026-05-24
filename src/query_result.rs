use std::{collections::HashMap, sync::Arc};

use pyo3::{
    prelude::*,
    pyclass, pymethods,
    types::{PyDict, PyIterator, PyList, PySlice, PyTuple},
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

    /// Return result as a list of `Record` instances.
    ///
    /// Each `Record` shares a column-descriptor (`RecordDesc`) with all other
    /// records from the same result set, avoiding per-row allocation of the
    /// column-name map. Values are eagerly decoded (like asyncpg `Record`).
    ///
    /// `result()` is unchanged — this method is additive.
    ///
    /// # Errors
    /// May return Err if a column value cannot be converted to Python.
    pub fn records(&self, py: Python<'_>) -> PSQLPyResult<Py<PyAny>> {
        if self.inner.is_empty() {
            return Ok(PyList::empty(py).into_py_any(py)?);
        }

        // Build the shared column descriptor from the first row's columns.
        let columns = self.inner[0].columns();
        let mut name_to_idx = HashMap::with_capacity(columns.len());
        let mut names = Vec::with_capacity(columns.len());
        for (i, col) in columns.iter().enumerate() {
            if name_to_idx.contains_key(col.name()) {
                return Err(crate::exceptions::rust_errors::RustPSQLDriverError::ConnectionExecuteError(
                    format!(
                        "Duplicate column name '{}' in result set; use positional indexing or aliases",
                        col.name()
                    ),
                ));
            }
            name_to_idx.insert(col.name().to_string(), i);
            names.push(col.name().to_string());
        }
        let desc = Arc::new(RecordDesc { name_to_idx, names });

        let mut records: Vec<Py<PyAny>> = Vec::with_capacity(self.inner.len());
        for row in &self.inner {
            let mut values: Vec<Py<PyAny>> = Vec::with_capacity(row.columns().len());
            for (idx, col) in row.columns().iter().enumerate() {
                values.push(postgres_to_py(py, row, col, idx, &None)?);
            }
            let record = Record {
                desc: Arc::clone(&desc),
                values,
            };
            records.push(Py::new(py, record)?.into_py_any(py)?);
        }
        Ok(records.into_py_any(py)?)
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

/// Shared column metadata for a result set. All `Record` instances from one
/// `records()` call point to the same `Arc<RecordDesc>`.
pub struct RecordDesc {
    name_to_idx: HashMap<String, usize>,
    names: Vec<String>,
}

/// An asyncpg-compatible row type: eagerly decoded values + shared column map.
///
/// Supports positional (`row[0]`) and by-name (`row["col"]`) access, iteration,
/// and dict-like `.keys()` / `.values()` / `.items()` / `.get()`.
#[pyclass(name = "Record")]
pub struct Record {
    desc: Arc<RecordDesc>,
    values: Vec<Py<PyAny>>,
}

#[pymethods]
impl Record {
    fn __len__(&self) -> usize {
        self.values.len()
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let fields: Vec<String> = self
            .desc
            .names
            .iter()
            .zip(self.values.iter())
            .map(|(name, val)| {
                let repr = val
                    .bind(py)
                    .repr()
                    .map_or_else(|_| "?".into(), |r| r.to_string());
                format!("{name}: {repr}")
            })
            .collect();
        format!("<Record {}>", fields.join(", "))
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn __getitem__(&self, py: Python<'_>, key: &pyo3::Bound<'_, PyAny>) -> PSQLPyResult<Py<PyAny>> {
        use crate::exceptions::rust_errors::RustPSQLDriverError;

        // Integer index
        if let Ok(idx) = key.extract::<isize>() {
            // Safe: len() <= isize::MAX on any platform we target
            let len = self.values.len() as isize;
            let real_idx = if idx < 0 { len + idx } else { idx };
            if real_idx < 0 || real_idx >= len {
                return Err(RustPSQLDriverError::RustPyError(
                    pyo3::exceptions::PyIndexError::new_err(format!(
                        "Record index {idx} out of range"
                    )),
                ));
            }
            // Safe: real_idx checked >= 0 above
            return Ok(self.values[real_idx as usize].clone_ref(py));
        }

        // Slice
        if let Ok(slice) = key.downcast::<PySlice>() {
            // Safe: len() <= isize::MAX on any platform we target
            let indices = slice.indices(self.values.len() as isize)?;
            let mut result: Vec<Py<PyAny>> = Vec::new();
            let mut i = indices.start;
            while (indices.step > 0 && i < indices.stop) || (indices.step < 0 && i > indices.stop) {
                // Safe: i is a valid index within the slice range
                result.push(self.values[i as usize].clone_ref(py));
                i += indices.step;
            }
            return Ok(result.into_py_any(py)?);
        }

        // String key
        if let Ok(name) = key.extract::<String>() {
            if let Some(&idx) = self.desc.name_to_idx.get(&name) {
                return Ok(self.values[idx].clone_ref(py));
            }
            return Err(RustPSQLDriverError::RustPyError(
                pyo3::exceptions::PyKeyError::new_err(name),
            ));
        }

        Err(RustPSQLDriverError::RustPyError(
            pyo3::exceptions::PyTypeError::new_err("Record key must be int, slice, or str"),
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PSQLPyResult<Py<PyAny>> {
        let list = PyList::new(py, self.values.iter().map(|v| v.clone_ref(py)))?;
        Ok(PyIterator::from_object(list.as_any())?.into_py_any(py)?)
    }

    #[pyo3(signature = (key, default=None))]
    fn get(&self, py: Python<'_>, key: &str, default: Option<Py<PyAny>>) -> Py<PyAny> {
        if let Some(&idx) = self.desc.name_to_idx.get(key) {
            self.values[idx].clone_ref(py)
        } else {
            default.unwrap_or_else(|| py.None())
        }
    }

    fn keys(&self) -> Vec<String> {
        self.desc.names.clone()
    }

    fn values(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        self.values.iter().map(|v| v.clone_ref(py)).collect()
    }

    fn items(&self, py: Python<'_>) -> Vec<(String, Py<PyAny>)> {
        self.desc
            .names
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| (k.clone(), v.clone_ref(py)))
            .collect()
    }
}
