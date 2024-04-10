use deadpool_postgres::Object;
use pyo3::{types::PyModule, PyAny, PyResult, Python};

use crate::{
    driver::transaction_options::{IsolationLevel, ReadVariant},
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
    parent_mod: &PyModule,
    child_mod_name: &'static str,
    child_mod_builder: impl FnOnce(Python<'_>, &PyModule) -> PyResult<()>,
) -> PyResult<()> {
    let sub_module = PyModule::new(py, child_mod_name)?;
    child_mod_builder(py, sub_module)?;
    parent_mod.add_submodule(sub_module)?;
    py.import("sys")?.getattr("modules")?.set_item(
        format!("{}.{}", parent_mod.name()?, child_mod_name),
        sub_module,
    )?;
    Ok(())
}

pub trait BaseTransactionQuery {
    fn start_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        defferable: Option<bool>,
    ) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
    fn commit(&self) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
    fn rollback(&self) -> impl std::future::Future<Output = RustPSQLDriverPyResult<()>> + Send;
}

impl BaseTransactionQuery for Object {
    async fn start_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> RustPSQLDriverPyResult<()> {
        let mut querystring = "START TRANSACTION".to_string();

        if let Some(level) = isolation_level {
            let level = &level.to_str_level();
            querystring.push_str(format!(" ISOLATION LEVEL {level}").as_str());
        };

        querystring.push_str(match read_variant {
            Some(ReadVariant::ReadOnly) => " READ ONLY",
            Some(ReadVariant::ReadWrite) => " READ WRITE",
            None => "",
        });

        querystring.push_str(match deferrable {
            Some(true) => " DEFERRABLE",
            Some(false) => " NOT DEFERRABLE",
            None => "",
        });
        self.batch_execute(&querystring).await?;

        Ok(())
    }
    async fn commit(&self) -> RustPSQLDriverPyResult<()> {
        self.batch_execute("COMMIT;").await?;
        Ok(())
    }
    async fn rollback(&self) -> RustPSQLDriverPyResult<()> {
        self.batch_execute("ROLLBACK;").await?;
        Ok(())
    }
}

pub trait BaseDataBaseQuery {
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

impl BaseDataBaseQuery for Object {
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
