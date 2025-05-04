use std::fmt::Display;

use regex::Regex;

use crate::value_converter::consts::KWARGS_PARAMS_REGEXP;

use super::utils::hash_str;

#[derive(Clone, Debug)]
pub struct QueryString {
    pub(crate) initial_qs: String,
    // This field are used when kwargs passed
    // from python side as parameters.
    pub(crate) converted_qs: Option<ConvertedQueryString>,
}

impl Display for QueryString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query())
    }
}

impl QueryString {
    pub fn new(initial_qs: &String) -> Self {
        return Self {
            initial_qs: initial_qs.clone(),
            converted_qs: None,
        };
    }

    pub(crate) fn query(&self) -> &str {
        if let Some(converted_qs) = &self.converted_qs {
            return converted_qs.query();
        }

        return &self.initial_qs;
    }

    pub(crate) fn hash(&self) -> u64 {
        hash_str(&self.initial_qs)
    }

    pub(crate) fn process_qs(&mut self) {
        if !self.is_kwargs_parametrized() {
            return ();
        }

        let mut counter = 0;
        let mut parameters_names = Vec::new();

        let re = Regex::new(KWARGS_PARAMS_REGEXP).unwrap();
        let result = re.replace_all(&self.initial_qs, |caps: &regex::Captures| {
            let parameter_idx = caps[1].to_string();

            parameters_names.push(parameter_idx.clone());
            counter += 1;

            format!("${}", &counter)
        });

        self.converted_qs = Some(ConvertedQueryString::new(result.into(), parameters_names));
    }

    fn is_kwargs_parametrized(&self) -> bool {
        Regex::new(KWARGS_PARAMS_REGEXP)
            .unwrap()
            .is_match(&self.initial_qs)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConvertedQueryString {
    converted_qs: String,
    params_names: Vec<String>,
}

impl ConvertedQueryString {
    fn new(converted_qs: String, params_names: Vec<String>) -> Self {
        Self {
            converted_qs,
            params_names,
        }
    }

    fn query(&self) -> &str {
        &self.converted_qs
    }

    pub(crate) fn params_names(&self) -> &Vec<String> {
        &self.params_names
    }
}
