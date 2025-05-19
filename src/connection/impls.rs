use bytes::Buf;
use pyo3::{PyAny, Python};
use tokio_postgres::{CopyInSink, Portal as tp_Portal, Row, Statement, ToStatement};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    options::{IsolationLevel, ReadVariant},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    statement::{statement::PsqlpyStatement, statement_builder::StatementBuilder},
    transaction::structs::PSQLPyTransaction,
    value_converter::to_python::postgres_to_py,
};

use deadpool_postgres::Transaction as dp_Transaction;
use tokio_postgres::Transaction as tp_Transaction;

use super::{
    structs::{PSQLPyConnection, PoolConnection, SingleConnection},
    traits::{CloseTransaction, Connection, StartTransaction, Transaction},
};

impl<T> Transaction for T
where
    T: Connection,
{
    async fn _start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> PSQLPyResult<()> {
        let start_qs = self.build_start_qs(isolation_level, read_variant, deferrable);
        self.batch_execute(start_qs.as_str()).await.map_err(|err| {
            RustPSQLDriverError::TransactionBeginError(
                format!("Cannot start transaction due to - {err}").into(),
            )
        })?;

        Ok(())
    }

    async fn _commit(&self) -> PSQLPyResult<()> {
        self.batch_execute("COMMIT;").await.map_err(|err| {
            RustPSQLDriverError::TransactionCommitError(format!(
                "Cannot execute COMMIT statement, error - {err}"
            ))
        })?;
        Ok(())
    }

    async fn _rollback(&self) -> PSQLPyResult<()> {
        self.batch_execute("ROLLBACK;").await.map_err(|err| {
            RustPSQLDriverError::TransactionRollbackError(format!(
                "Cannot execute ROLLBACK statement, error - {err}"
            ))
        })?;
        Ok(())
    }
}

impl Connection for SingleConnection {
    async fn prepare(&self, query: &str, prepared: bool) -> PSQLPyResult<Statement> {
        let prepared_stmt = self.connection.prepare(query).await?;

        if !prepared {
            self.drop_prepared(&prepared_stmt).await?;
        }
        return Ok(prepared_stmt);
    }

    async fn drop_prepared(&self, stmt: &Statement) -> PSQLPyResult<()> {
        let deallocate_query = format!("DEALLOCATE PREPARE {}", stmt.name());

        Ok(self.connection.batch_execute(&deallocate_query).await?)
    }

    async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<Vec<Row>>
    where
        T: ?Sized + ToStatement,
    {
        Ok(self.connection.query(statement, params).await?)
    }

    async fn query_typed(
        &self,
        statement: &str,
        params: &[(&(dyn postgres_types::ToSql + Sync), postgres_types::Type)],
    ) -> PSQLPyResult<Vec<Row>> {
        Ok(self.connection.query_typed(statement, params).await?)
    }

    async fn batch_execute(&self, query: &str) -> PSQLPyResult<()> {
        Ok(self.connection.batch_execute(query).await?)
    }

    async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<Row>
    where
        T: ?Sized + ToStatement,
    {
        Ok(self.connection.query_one(statement, params).await?)
    }
}

impl StartTransaction for SingleConnection {
    async fn start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> PSQLPyResult<()> {
        let res = self
            ._start_transaction(isolation_level, read_variant, deferrable)
            .await?;
        self.in_transaction = true;

        Ok(res)
    }
}

impl CloseTransaction for SingleConnection {
    async fn commit(&mut self) -> PSQLPyResult<()> {
        let res = self._commit().await?;
        self.in_transaction = false;

        Ok(res)
    }

    async fn rollback(&mut self) -> PSQLPyResult<()> {
        let res = self._rollback().await?;
        self.in_transaction = false;

        Ok(res)
    }
}

impl Connection for PoolConnection {
    async fn prepare(&self, query: &str, prepared: bool) -> PSQLPyResult<Statement> {
        if prepared {
            return Ok(self.connection.prepare_cached(query).await?);
        }

        let prepared = self.connection.prepare(query).await?;
        self.drop_prepared(&prepared).await?;
        return Ok(prepared);
    }

    async fn drop_prepared(&self, stmt: &Statement) -> PSQLPyResult<()> {
        let deallocate_query = format!("DEALLOCATE PREPARE {}", stmt.name());

        Ok(self.connection.batch_execute(&deallocate_query).await?)
    }

    async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<Vec<Row>>
    where
        T: ?Sized + ToStatement,
    {
        Ok(self.connection.query(statement, params).await?)
    }

    async fn query_typed(
        &self,
        statement: &str,
        params: &[(&(dyn postgres_types::ToSql + Sync), postgres_types::Type)],
    ) -> PSQLPyResult<Vec<Row>> {
        Ok(self.connection.query_typed(statement, params).await?)
    }

    async fn batch_execute(&self, query: &str) -> PSQLPyResult<()> {
        Ok(self.connection.batch_execute(query).await?)
    }

    async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<Row>
    where
        T: ?Sized + ToStatement,
    {
        Ok(self.connection.query_one(statement, params).await?)
    }
}

impl StartTransaction for PoolConnection {
    async fn start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> PSQLPyResult<()> {
        self.in_transaction = true;
        self._start_transaction(isolation_level, read_variant, deferrable)
            .await
    }
}

impl CloseTransaction for PoolConnection {
    async fn commit(&mut self) -> PSQLPyResult<()> {
        let res = self._commit().await?;
        self.in_transaction = false;

        Ok(res)
    }

    async fn rollback(&mut self) -> PSQLPyResult<()> {
        let res = self._rollback().await?;
        self.in_transaction = false;

        Ok(res)
    }
}

impl Connection for PSQLPyConnection {
    async fn prepare(&self, query: &str, prepared: bool) -> PSQLPyResult<Statement> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.prepare(query, prepared).await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.prepare(query, prepared).await,
        }
    }

    async fn drop_prepared(&self, stmt: &Statement) -> PSQLPyResult<()> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.drop_prepared(stmt).await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.drop_prepared(stmt).await,
        }
    }

    async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<Vec<Row>>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.query(statement, params).await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.query(statement, params).await,
        }
    }

    async fn query_typed(
        &self,
        statement: &str,
        params: &[(&(dyn postgres_types::ToSql + Sync), postgres_types::Type)],
    ) -> PSQLPyResult<Vec<Row>> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.query_typed(statement, params).await,
            PSQLPyConnection::SingleConnection(s_conn) => {
                s_conn.query_typed(statement, params).await
            }
        }
    }

    async fn batch_execute(&self, query: &str) -> PSQLPyResult<()> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.batch_execute(query).await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.batch_execute(query).await,
        }
    }

    async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<Row>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.query_one(statement, params).await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.query_one(statement, params).await,
        }
    }
}

impl StartTransaction for PSQLPyConnection {
    async fn start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> PSQLPyResult<()> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => {
                p_conn
                    .start_transaction(isolation_level, read_variant, deferrable)
                    .await
            }
            PSQLPyConnection::SingleConnection(s_conn) => {
                s_conn
                    .start_transaction(isolation_level, read_variant, deferrable)
                    .await
            }
        }
    }
}

impl CloseTransaction for PSQLPyConnection {
    async fn commit(&mut self) -> PSQLPyResult<()> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.commit().await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.commit().await,
        }
    }

    async fn rollback(&mut self) -> PSQLPyResult<()> {
        match self {
            PSQLPyConnection::PoolConn(p_conn) => p_conn.rollback().await,
            PSQLPyConnection::SingleConnection(s_conn) => s_conn.rollback().await,
        }
    }
}

impl PSQLPyConnection {
    pub fn in_transaction(&self) -> bool {
        match self {
            PSQLPyConnection::PoolConn(conn) => conn.in_transaction,
            PSQLPyConnection::SingleConnection(conn) => conn.in_transaction,
        }
    }

    pub async fn prepare_statement(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
    ) -> PSQLPyResult<PsqlpyStatement> {
        StatementBuilder::new(&querystring, &parameters, self, Some(true))
            .build()
            .await
    }

    pub async fn execute_statement(
        &self,
        statement: &PsqlpyStatement,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let result = self
            .query(statement.statement_query()?, &statement.params())
            .await?;

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    pub async fn execute(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let statement = StatementBuilder::new(&querystring, &parameters, self, prepared)
            .build()
            .await?;

        let prepared = prepared.unwrap_or(true);
        let result = match prepared {
            true => {
                self.query(statement.statement_query()?, &statement.params())
                    .await
            }
            false => {
                self.query_typed(statement.raw_query(), &statement.params_typed())
                    .await
            }
        };

        let return_result = result.map_err(|err| {
            RustPSQLDriverError::ConnectionExecuteError(format!(
                "Cannot execute query, error - {err}"
            ))
        })?;

        Ok(PSQLDriverPyQueryResult::new(return_result))
    }

    pub async fn execute_many(
        &self,
        querystring: String,
        parameters: Option<Vec<pyo3::Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<()> {
        let mut statements: Vec<PsqlpyStatement> = vec![];
        if let Some(parameters) = parameters {
            for vec_of_py_any in parameters {
                // TODO: Fix multiple qs creation
                let statement =
                    StatementBuilder::new(&querystring, &Some(vec_of_py_any), self, prepared)
                        .build()
                        .await?;

                statements.push(statement);
            }
        }

        let prepared = prepared.unwrap_or(true);

        for statement in statements {
            let querystring_result = if prepared {
                let prepared_stmt = &self.prepare(&statement.raw_query(), true).await;
                if let Err(error) = prepared_stmt {
                    return Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                        "Cannot prepare statement in execute_many, operation rolled back {error}",
                    )));
                }
                self.query(
                    &self.prepare(&statement.raw_query(), true).await?,
                    &statement.params(),
                )
                .await
            } else {
                self.query(statement.raw_query(), &statement.params()).await
            };

            if let Err(error) = querystring_result {
                return Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Error occured in `execute_many` statement: {error}"
                )));
            }
        }

        return Ok(());
    }

    pub async fn fetch_row_raw(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<Row> {
        let statement = StatementBuilder::new(&querystring, &parameters, self, prepared)
            .build()
            .await?;

        let prepared = prepared.unwrap_or(true);

        let result = if prepared {
            self.query_one(
                &self
                    .prepare(&statement.raw_query(), true)
                    .await
                    .map_err(|err| {
                        RustPSQLDriverError::ConnectionExecuteError(format!(
                            "Cannot prepare statement, error - {err}"
                        ))
                    })?,
                &statement.params(),
            )
            .await
            .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        } else {
            self.query_one(statement.raw_query(), &statement.params())
                .await
                .map_err(|err| RustPSQLDriverError::ConnectionExecuteError(format!("{err}")))?
        };

        return Ok(result);
    }

    pub async fn fetch_row(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverSinglePyQueryResult> {
        let result = self
            .fetch_row_raw(querystring, parameters, prepared)
            .await?;

        return Ok(PSQLDriverSinglePyQueryResult::new(result));
    }

    pub async fn fetch_val(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<pyo3::Py<PyAny>> {
        let result = self
            .fetch_row_raw(querystring, parameters, prepared)
            .await?;

        return Python::with_gil(|gil| match result.columns().first() {
            Some(first_column) => postgres_to_py(gil, &result, first_column, 0, &None),
            None => Ok(gil.None()),
        });
    }

    pub async fn copy_in<T, U>(&self, statement: &T) -> PSQLPyResult<CopyInSink<U>>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        match self {
            PSQLPyConnection::PoolConn(pconn) => {
                return Ok(pconn.connection.copy_in(statement).await?)
            }
            PSQLPyConnection::SingleConnection(sconn) => {
                return Ok(sconn.connection.copy_in(statement).await?)
            }
        }
    }

    pub async fn transaction(&mut self) -> PSQLPyResult<PSQLPyTransaction> {
        match self {
            PSQLPyConnection::PoolConn(conn) => {
                let transaction = unsafe {
                    std::mem::transmute::<dp_Transaction<'_>, dp_Transaction<'static>>(
                        conn.connection.transaction().await?,
                    )
                };
                Ok(PSQLPyTransaction::PoolTransaction(transaction))
            }
            PSQLPyConnection::SingleConnection(conn) => {
                let transaction = unsafe {
                    std::mem::transmute::<tp_Transaction<'_>, tp_Transaction<'static>>(
                        conn.connection.transaction().await?,
                    )
                };
                Ok(PSQLPyTransaction::SingleTransaction(transaction))
            }
        }
    }

    pub async fn portal(
        &mut self,
        querystring: Option<&String>,
        parameters: &Option<pyo3::Py<PyAny>>,
        statement: Option<&PsqlpyStatement>,
    ) -> PSQLPyResult<(PSQLPyTransaction, tp_Portal)> {
        let statement = {
            match statement {
                Some(stmt) => stmt,
                None => {
                    let Some(querystring) = querystring else {
                        return Err(RustPSQLDriverError::ConnectionExecuteError(
                            "Can't create cursor without querystring".into(),
                        ));
                    };

                    &StatementBuilder::new(querystring, parameters, self, Some(false))
                        .build()
                        .await?
                }
            }
        };

        let transaction = self.transaction().await?;
        let inner_portal = transaction
            .portal(statement.raw_query(), &statement.params())
            .await?;

        Ok((transaction, inner_portal))
    }
}
