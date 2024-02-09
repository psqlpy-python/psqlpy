pub mod common;
pub mod engine;
pub mod exceptions;
pub mod extra_types;
pub mod query_result;
pub mod value_converter;

use common::add_module;
use extra_types::extra_types_module;
use pyo3::{pymodule, types::PyModule, PyResult, Python};

#[pymodule]
#[pyo3(name = "_internal")]
fn psql_rust_engine(py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<engine::PSQLPool>()?;
    pymod.add_class::<engine::Transaction>()?;
    pymod.add_class::<query_result::PSQLDriverPyQueryResult>()?;
    add_module(py, pymod, "extra_types", extra_types_module)?;
    Ok(())
}
