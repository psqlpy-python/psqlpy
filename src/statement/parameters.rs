use std::iter::zip;

use postgres_types::{ToSql, Type};
use pyo3::{
    conversion::FromPyObjectBound,
    types::{PyAnyMethods, PyMapping},
    Py, PyObject, PyTypeCheck, Python,
};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    value_converter::{
        dto::enums::PythonDTO,
        from_python::{from_python_typed, from_python_untyped},
    },
};

pub type QueryParameter = (dyn ToSql + Sync);

pub(crate) struct ParametersBuilder {
    parameters: Option<PyObject>,
    types: Option<Vec<Type>>,
}

impl ParametersBuilder {
    pub fn new(parameters: &Option<PyObject>, types: Option<Vec<Type>>) -> Self {
        Self {
            parameters: parameters.clone(),
            types,
        }
    }

    pub fn prepare(
        self,
        parameters_names: Option<Vec<String>>,
    ) -> PSQLPyResult<PreparedParameters> {
        let prepared_parameters =
            Python::with_gil(|gil| self.prepare_parameters(gil, parameters_names))?;

        Ok(prepared_parameters)
    }

    fn prepare_parameters(
        self,
        gil: Python<'_>,
        parameters_names: Option<Vec<String>>,
    ) -> PSQLPyResult<PreparedParameters> {
        if self.parameters.is_none() {
            return Ok(PreparedParameters::default());
        }

        let sequence_typed = self.as_type::<Vec<PyObject>>(gil);
        let mapping_typed = self.downcast_as::<PyMapping>(gil);
        let mut prepared_parameters: Option<PreparedParameters> = None;

        match (sequence_typed, mapping_typed) {
            (Some(sequence), None) => {
                prepared_parameters =
                    Some(SequenceParametersBuilder::new(sequence, self.types).prepare(gil)?);
            }
            (None, Some(mapping)) => {
                if let Some(parameters_names) = parameters_names {
                    prepared_parameters = Some(
                        MappingParametersBuilder::new(mapping, self.types)
                            .prepare(gil, parameters_names)?,
                    )
                }
            }
            _ => {}
        }

        if let Some(prepared_parameters) = prepared_parameters {
            return Ok(prepared_parameters);
        }

        return Err(RustPSQLDriverError::PyToRustValueConversionError(
            "Parameters must be sequence or mapping".into(),
        ));
    }

    fn as_type<T: for<'a, 'py> FromPyObjectBound<'a, 'py>>(&self, gil: Python<'_>) -> Option<T> {
        if let Some(parameters) = &self.parameters {
            let extracted_param = parameters.extract::<T>(gil);

            if let Ok(extracted_param) = extracted_param {
                return Some(extracted_param);
            }

            return None;
        }

        None
    }

    fn downcast_as<T: PyTypeCheck>(&self, gil: Python<'_>) -> Option<Py<T>> {
        if let Some(parameters) = &self.parameters {
            let extracted_param = parameters.downcast_bound::<T>(gil);

            if let Ok(extracted_param) = extracted_param {
                return Some(extracted_param.clone().unbind());
            }

            return None;
        }

        None
    }
}

pub(crate) struct MappingParametersBuilder {
    map_parameters: Py<PyMapping>,
    types: Option<Vec<Type>>,
}

impl MappingParametersBuilder {
    fn new(map_parameters: Py<PyMapping>, types: Option<Vec<Type>>) -> Self {
        Self {
            map_parameters,
            types,
        }
    }

    fn prepare(
        self,
        gil: Python<'_>,
        parameters_names: Vec<String>,
    ) -> PSQLPyResult<PreparedParameters> {
        if self.types.is_some() {
            return self.prepare_typed(gil, parameters_names);
        }

        self.prepare_not_typed(gil, parameters_names)
    }

    fn prepare_typed(
        self,
        gil: Python<'_>,
        parameters_names: Vec<String>,
    ) -> PSQLPyResult<PreparedParameters> {
        let converted_parameters = self
            .extract_parameters(gil, parameters_names)?
            .iter()
            .map(|parameter| from_python_untyped(parameter.bind(gil)))
            .collect::<PSQLPyResult<Vec<PythonDTO>>>()?;

        Ok(PreparedParameters::new(converted_parameters, vec![])) // TODO: change vec![] to real types.
    }

    fn prepare_not_typed(
        self,
        gil: Python<'_>,
        parameters_names: Vec<String>,
    ) -> PSQLPyResult<PreparedParameters> {
        let converted_parameters = self
            .extract_parameters(gil, parameters_names)?
            .iter()
            .map(|parameter| from_python_untyped(parameter.bind(gil)))
            .collect::<PSQLPyResult<Vec<PythonDTO>>>()?;

        Ok(PreparedParameters::new(converted_parameters, vec![])) // TODO: change vec![] to real types.
    }

    fn extract_parameters(
        &self,
        gil: Python<'_>,
        parameters_names: Vec<String>,
    ) -> PSQLPyResult<Vec<PyObject>> {
        let mut params_as_pyobject: Vec<PyObject> = vec![];

        for param_name in parameters_names {
            match self.map_parameters.bind(gil).get_item(&param_name) {
                Ok(param_value) => params_as_pyobject.push(param_value.unbind()),
                Err(_) => {
                    return Err(RustPSQLDriverError::PyToRustValueConversionError(
                        format!("Cannot find parameter with name <{}>", param_name).into(),
                    ))
                }
            }
        }

        Ok(params_as_pyobject)
    }
}

pub(crate) struct SequenceParametersBuilder {
    seq_parameters: Vec<PyObject>,
    types: Option<Vec<Type>>,
}

impl SequenceParametersBuilder {
    fn new(seq_parameters: Vec<PyObject>, types: Option<Vec<Type>>) -> Self {
        Self {
            seq_parameters: seq_parameters,
            types,
        }
    }

    fn prepare(self, gil: Python<'_>) -> PSQLPyResult<PreparedParameters> {
        let types = self.types.clone();

        if types.is_some() {
            return self.prepare_typed(gil, types.clone().unwrap());
        }

        self.prepare_not_typed(gil)
    }

    fn prepare_typed(self, gil: Python<'_>, types: Vec<Type>) -> PSQLPyResult<PreparedParameters> {
        let zipped_params_types = zip(self.seq_parameters, &types);
        let converted_parameters = zipped_params_types
            .map(|(parameter, type_)| from_python_typed(parameter.bind(gil), &type_))
            .collect::<PSQLPyResult<Vec<PythonDTO>>>()?;

        Ok(PreparedParameters::new(converted_parameters, types))
    }

    fn prepare_not_typed(self, gil: Python<'_>) -> PSQLPyResult<PreparedParameters> {
        let converted_parameters = self
            .seq_parameters
            .iter()
            .map(|parameter| from_python_untyped(parameter.bind(gil)))
            .collect::<PSQLPyResult<Vec<PythonDTO>>>()?;

        Ok(PreparedParameters::new(converted_parameters, vec![])) // TODO: change vec![] to real types.
    }
}

#[derive(Default, Clone, Debug)]
pub struct PreparedParameters {
    parameters: Vec<PythonDTO>,
    types: Vec<Type>,
}

impl PreparedParameters {
    pub fn new(parameters: Vec<PythonDTO>, types: Vec<Type>) -> Self {
        Self { parameters, types }
    }

    pub fn params(&self) -> Box<[&(dyn ToSql + Sync)]> {
        let params_ref = &self.parameters;
        params_ref
            .iter()
            .map(|param| param as &QueryParameter)
            .collect::<Vec<&QueryParameter>>()
            .into_boxed_slice()
    }

    pub fn params_typed(&self) -> Box<[(&(dyn ToSql + Sync), Type)]> {
        let params_ref = &self.parameters;
        let types = self.types.clone();
        let params_types = zip(params_ref, types);
        params_types
            .map(|(param, type_)| (param as &QueryParameter, type_))
            .collect::<Vec<(&QueryParameter, Type)>>()
            .into_boxed_slice()
    }
}
