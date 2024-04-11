use deadpool_postgres::Object;
use pyo3::{
    types::{PyAnyMethods, PyModule, PyModuleMethods},
    Bound, PyAny, PyResult, Python,
};

use crate::{
    exceptions::rust_errors::RustPSQLDriverPyResult,
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    value_converter::{convert_parameters, PythonDTO, QueryParameter},
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
    let sub_module = PyModule::new_bound(py, child_mod_name)?;
    child_mod_builder(py, &sub_module)?;
    parent_mod.add_submodule(&sub_module)?;
    py.import_bound("sys")?.getattr("modules")?.set_item(
        format!("{}.{}", parent_mod.name()?, child_mod_name),
        sub_module,
    )?;
    Ok(())
}

pub trait ObjectQueryTrait {
    fn psqlpy_query_one(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> impl std::future::Future<Output = RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult>> + Send;

    fn psqlpy_query(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> impl std::future::Future<Output = RustPSQLDriverPyResult<PSQLDriverPyQueryResult>> + Send;

    fn psqlpy_query_simple(
        &self,
        querystring: String,
    ) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
}

impl ObjectQueryTrait for Object {
    async fn psqlpy_query_one(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverSinglePyQueryResult> {
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            self.query_one(
                &self.prepare_cached(&querystring).await?,
                &params
                    .iter()
                    .map(|param| param as &QueryParameter)
                    .collect::<Vec<&QueryParameter>>()
                    .into_boxed_slice(),
            )
            .await?
        } else {
            self.query_one(
                &querystring,
                &params
                    .iter()
                    .map(|param| param as &QueryParameter)
                    .collect::<Vec<&QueryParameter>>()
                    .into_boxed_slice(),
            )
            .await?
        };

        Ok(PSQLDriverSinglePyQueryResult::new(result))
    }

    async fn psqlpy_query(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> RustPSQLDriverPyResult<PSQLDriverPyQueryResult> {
        let mut params: Vec<PythonDTO> = vec![];
        if let Some(parameters) = parameters {
            params = convert_parameters(parameters)?;
        }
        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            self.query(
                &self.prepare_cached(&querystring).await?,
                &params
                    .iter()
                    .map(|param| param as &QueryParameter)
                    .collect::<Vec<&QueryParameter>>()
                    .into_boxed_slice(),
            )
            .await?
        } else {
            self.query(
                &querystring,
                &params
                    .iter()
                    .map(|param| param as &QueryParameter)
                    .collect::<Vec<&QueryParameter>>()
                    .into_boxed_slice(),
            )
            .await?
        };

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    async fn psqlpy_query_simple(&self, querystring: String) -> RustPSQLDriverPyResult<()> {
        Ok(self.batch_execute(querystring.as_str()).await?)
    }
}
