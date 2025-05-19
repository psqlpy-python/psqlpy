pub mod common;
pub mod connection;
pub mod driver;
pub mod exceptions;
pub mod extra_types;
pub mod format_helpers;
pub mod options;
pub mod query_result;
pub mod row_factories;
pub mod runtime;
pub mod statement;
pub mod transaction;
pub mod value_converter;

use common::add_module;
use exceptions::python_errors::python_exceptions_module;
use extra_types::extra_types_module;
use pyo3::{
    pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult, Python,
};
use row_factories::row_factories_module;

#[pymodule]
#[pyo3(name = "_internal")]
fn psqlpy(py: Python<'_>, pymod: &Bound<'_, PyModule>) -> PyResult<()> {
    pymod.add_class::<driver::connection_pool::ConnectionPool>()?;
    pymod.add_class::<driver::connection_pool::ConnectionPoolStatus>()?;
    pymod.add_class::<driver::connection_pool_builder::ConnectionPoolBuilder>()?;
    pymod.add_function(wrap_pyfunction!(
        driver::connection_pool::connect_pool,
        pymod
    )?)?;
    pymod.add_class::<driver::connection::Connection>()?;
    pymod.add_function(wrap_pyfunction!(driver::connection::connect, pymod)?)?;
    pymod.add_class::<driver::transaction::Transaction>()?;
    // pymod.add_class::<driver::cursor::Cursor>()?;
    pymod.add_class::<statement::parameters::Column>()?;
    pymod.add_class::<driver::prepared_statement::PreparedStatement>()?;
    pymod.add_class::<driver::cursor::Cursor>()?;
    pymod.add_class::<driver::listener::core::Listener>()?;
    pymod.add_class::<driver::listener::structs::ListenerNotificationMsg>()?;
    pymod.add_class::<options::IsolationLevel>()?;
    pymod.add_class::<options::ReadVariant>()?;
    pymod.add_class::<options::ConnRecyclingMethod>()?;
    pymod.add_class::<options::LoadBalanceHosts>()?;
    pymod.add_class::<options::TargetSessionAttrs>()?;
    pymod.add_class::<options::SslMode>()?;
    pymod.add_class::<options::KeepaliveConfig>()?;
    pymod.add_class::<query_result::PSQLDriverPyQueryResult>()?;
    pymod.add_class::<query_result::PSQLDriverSinglePyQueryResult>()?;
    add_module(py, pymod, "extra_types", extra_types_module)?;
    add_module(py, pymod, "exceptions", python_exceptions_module)?;
    add_module(py, pymod, "row_factories", row_factories_module)?;
    Ok(())
}
