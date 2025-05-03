use postgres_types::{ToSql, Type};

use super::{parameters::PreparedParameters, query::QueryString};

#[derive(Clone)]
pub struct PsqlpyStatement {
    query: QueryString,
    prepared_parameters: PreparedParameters,
}

impl PsqlpyStatement {
    pub(crate) fn new(query: QueryString, prepared_parameters: PreparedParameters) -> Self {
        Self {
            query,
            prepared_parameters,
        }
    }

    pub fn sql_stmt(&self) -> &str {
        self.query.query()
    }

    pub fn params(&self) -> Box<[&(dyn ToSql + Sync)]> {
        self.prepared_parameters.params()
    }

    pub fn params_typed(&self) -> Box<[(&(dyn ToSql + Sync), Type)]> {
        self.prepared_parameters.params_typed()
    }
}
