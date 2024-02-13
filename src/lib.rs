pub mod common;
pub mod driver;
pub mod exceptions;
pub mod extra_types;
pub mod query_result;
pub mod value_converter;

use common::add_module;
use exceptions::python_errors::python_exceptions_module;
use extra_types::extra_types_module;
use pyo3::{pymodule, types::PyModule, PyResult, Python};

#[pymodule]
#[pyo3(name = "_internal")]
fn psql_rust_engine(py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<driver::connection_pool::PSQLPool>()?;
    pymod.add_class::<driver::transaction::Transaction>()?;
    pymod.add_class::<driver::transaction_options::IsolationLevel>()?;
    pymod.add_class::<query_result::PSQLDriverPyQueryResult>()?;
    add_module(py, pymod, "extra_types", extra_types_module)?;
    add_module(py, pymod, "exceptions", python_exceptions_module)?;
    Ok(())
}
