use pyo3::{
    types::{PyAnyMethods, PyModule, PyModuleMethods},
    Bound, PyResult, Python,
};

/// Add new module to the parent one.
///
/// You can find out more information from this issue
/// <https://github.com/PyO3/pyo3/issues/759>
///
/// # Errors
///
/// May return Err Result if can't build module or change modules.
pub fn add_module(
    py: Python<'_>,
    parent_mod: &Bound<'_, PyModule>,
    child_mod_name: &'static str,
    child_mod_builder: impl FnOnce(Python<'_>, &Bound<'_, PyModule>) -> PyResult<()>,
) -> PyResult<()> {
    let sub_module = PyModule::new(py, child_mod_name)?;
    child_mod_builder(py, &sub_module)?;
    parent_mod.add_submodule(&sub_module)?;
    py.import("sys")?.getattr("modules")?.set_item(
        format!("{}.{}", parent_mod.name()?, child_mod_name),
        sub_module,
    )?;
    Ok(())
}
