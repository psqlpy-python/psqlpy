pub mod additional_types;
pub mod common;
pub mod driver;
pub mod exceptions;
pub mod extra_types;
pub mod query_result;
pub mod runtime;
pub mod value_converter;

use common::add_module;
use exceptions::python_errors::python_exceptions_module;
use extra_types::extra_types_module;
use pyo3::{pymodule, types::PyModule, wrap_pyfunction, Bound, PyResult, Python};

#[pymodule]
#[pyo3(name = "_internal")]
fn psqlpy(py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add_class::<driver::connection_pool::ConnectionPool>()?;
    pymod.add_function(wrap_pyfunction!(
        driver::connection_pool::create_pool,
        pymod
    )?)?;
    pymod.add_class::<driver::connection::Connection>()?;
    pymod.add_function(wrap_pyfunction!(driver::connection::connect, pymod)?)?;
    pymod.add_class::<driver::transaction::Transaction>()?;
    pymod.add_class::<driver::cursor::Cursor>()?;
    pymod.add_class::<driver::transaction_options::IsolationLevel>()?;
    pymod.add_class::<driver::transaction_options::ReadVariant>()?;
    pymod.add_class::<driver::common_options::ConnRecyclingMethod>()?;
    pymod.add_class::<query_result::PSQLDriverPyQueryResult>()?;
    pymod.add_class::<query_result::PSQLDriverSinglePyQueryResult>()?;
    add_module(py, pymod, "extra_types", extra_types_module)?;
    add_module(py, pymod, "exceptions", python_exceptions_module)?;
    Ok(())
}
