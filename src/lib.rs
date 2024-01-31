pub mod engine;
pub mod value_converter;

use pyo3::{pymodule, types::PyModule, PyResult, Python};

#[pymodule]
fn rustengine(py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<engine::RustEngine>()?;
    Ok(())
}
