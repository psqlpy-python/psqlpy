use deadpool_postgres::Object;
use postgres_types::ToSql;
use tokio_postgres::Client;

struct CConnection<'a>(&'a dyn BaseConnection);

pub trait BaseConnection {
    async fn prepare_stmt_cached(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error>;
    async fn query_qs<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement;
    async fn query_qs_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement;
}

impl BaseConnection for Object {
    async fn prepare_stmt_cached(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error> {
        self.prepare_cached(query).await
    }

    async fn query_qs<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement,
    {
        self.query(statement, params).await
    }

    async fn query_qs_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement,
    {
        self.query_one(statement, params).await
    }
}

impl BaseConnection for Client {
    async fn prepare_stmt_cached(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error> {
        self.prepare(query).await
    }

    async fn query_qs<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement,
    {
        self.query(statement, params).await
    }

    async fn query_qs_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement,
    {
        self.query_one(statement, params).await
    }
}
