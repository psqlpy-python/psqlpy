pub mod engine;
pub mod query_result;
pub mod value_converter;

use pyo3::{pymodule, types::PyModule, PyResult, Python};

#[pymodule]
fn rustengine(_py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<engine::RustEngine>()?;
    Ok(())
}
