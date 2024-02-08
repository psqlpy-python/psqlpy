pub mod engine;
pub mod exceptions;
pub mod extra_types;
pub mod query_result;
pub mod value_converter;

use pyo3::{pymodule, types::PyModule, PyResult, Python};

#[pymodule]
#[pyo3(name = "_internal")]
fn psql_rust_engine(_py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<engine::PSQLPool>()?;
    pymod.add_class::<engine::Transaction>()?;
    pymod.add_class::<query_result::PSQLDriverPyQueryResult>()?;

    pymod.add_class::<extra_types::SmallInt>()?;
    pymod.add_class::<extra_types::Integer>()?;
    pymod.add_class::<extra_types::BigInt>()?;
    Ok(())
}
