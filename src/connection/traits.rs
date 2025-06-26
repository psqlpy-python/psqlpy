use postgres_types::{ToSql, Type};
use tokio_postgres::{Row, Statement, ToStatement};

use crate::exceptions::rust_errors::PSQLPyResult;

use crate::options::{IsolationLevel, ReadVariant};

pub trait Connection {
    fn prepare(
        &self,
        query: &str,
        prepared: bool,
    ) -> impl std::future::Future<Output = PSQLPyResult<Statement>> + Send;

    fn drop_prepared(
        &self,
        stmt: &Statement,
    ) -> impl std::future::Future<Output = PSQLPyResult<()>> + Send;

    fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> impl std::future::Future<Output = PSQLPyResult<Vec<Row>>>
    where
        T: ?Sized + ToStatement;

    fn query_typed(
        &self,
        statement: &str,
        params: &[(&(dyn ToSql + Sync), Type)],
    ) -> impl std::future::Future<Output = PSQLPyResult<Vec<Row>>>;

    fn batch_execute(
        &self,
        query: &str,
    ) -> impl std::future::Future<Output = PSQLPyResult<()>> + Send;

    fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> impl std::future::Future<Output = PSQLPyResult<Row>>
    where
        T: ?Sized + ToStatement;
}

pub trait Transaction {
    fn build_start_qs(
        &self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> String {
        let mut querystring = "START TRANSACTION".to_string();

        if let Some(level) = isolation_level {
            let level = &level.to_str_level();
            querystring.push_str(format!(" ISOLATION LEVEL {level}").as_str());
        }

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

        querystring
    }

    fn _start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> impl std::future::Future<Output = PSQLPyResult<()>>;

    fn _commit(&self) -> impl std::future::Future<Output = PSQLPyResult<()>>;

    fn _rollback(&self) -> impl std::future::Future<Output = PSQLPyResult<()>>;
}

pub trait StartTransaction: Transaction {
    fn start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> impl std::future::Future<Output = PSQLPyResult<()>>;
}

pub trait CloseTransaction: StartTransaction {
    fn commit(&mut self) -> impl std::future::Future<Output = PSQLPyResult<()>>;

    fn rollback(&mut self) -> impl std::future::Future<Output = PSQLPyResult<()>>;
}
