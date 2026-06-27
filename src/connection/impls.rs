use bytes::Buf;
use futures::stream::{FuturesOrdered, Stream, StreamExt};
use postgres_types::{ToSql, Type};
use pyo3::{PyAny, Python};
use tokio_postgres::{CopyInSink, Portal as tp_Portal, Row, Statement, ToStatement};

use crate::{
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    options::{IsolationLevel, ReadVariant},
    query_result::{PSQLDriverPyQueryResult, PSQLDriverSinglePyQueryResult},
    statement::{
        parameters::ParametersBuilder, statement::PsqlpyStatement,
        statement_builder::StatementBuilder,
    },
    transaction::structs::PSQLPyTransaction,
    value_converter::to_python::postgres_to_py,
};

use deadpool_postgres::Transaction as dp_Transaction;
use tokio_postgres::Transaction as tp_Transaction;

use super::{
    structs::{CopyTypeCache, PSQLPyConnection, PoolConnection, SingleConnection},
    traits::{CloseTransaction, Connection, StartTransaction, Transaction},
};

/// Drain a `FuturesOrdered` stream to completion, returning the first error seen.
///
/// All futures are polled to completion regardless of errors — this keeps
/// the underlying connection in a quiescent state before the caller issues
/// the next statement (e.g., ROLLBACK TO SAVEPOINT after a failed batch).
async fn drain_ordered<T>(
    mut stream: impl Stream<Item = PSQLPyResult<T>> + Unpin,
) -> PSQLPyResult<()> {
    let mut first_err: Option<RustPSQLDriverError> = None;
    while let Some(res) = stream.next().await {
        if let Err(err) = res {
            if first_err.is_none() {
                first_err = Some(RustPSQLDriverError::ConnectionExecuteError(format!(
                    "Error occurred in `execute_many` statement: {err}"
                )));
            }
        }
    }
    match first_err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

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
            RustPSQLDriverError::TransactionBeginError(format!(
                "Cannot start transaction due to - {err}"
            ))
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
        if prepared {
            if let Some(cached) = self.stmt_cache.get(query) {
                return Ok(cached.clone());
            }
            let stmt = self.connection.prepare(query).await?;
            self.stmt_cache.insert(query.to_string(), stmt.clone());
            return Ok(stmt);
        }

        let stmt = self.connection.prepare(query).await?;
        self.drop_prepared(&stmt).await?;
        Ok(stmt)
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
    #[allow(clippy::used_underscore_items)]
    async fn start_transaction(
        &mut self,
        isolation_level: Option<IsolationLevel>,
        read_variant: Option<ReadVariant>,
        deferrable: Option<bool>,
    ) -> PSQLPyResult<()> {
        self._start_transaction(isolation_level, read_variant, deferrable)
            .await?;
        self.in_transaction = true;

        Ok(())
    }
}

impl CloseTransaction for SingleConnection {
    #[allow(clippy::used_underscore_items)]
    async fn commit(&mut self) -> PSQLPyResult<()> {
        self._commit().await?;
        self.in_transaction = false;

        Ok(())
    }

    #[allow(clippy::used_underscore_items)]
    async fn rollback(&mut self) -> PSQLPyResult<()> {
        self._rollback().await?;
        self.in_transaction = false;

        Ok(())
    }
}

impl Connection for PoolConnection {
    async fn prepare(&self, query: &str, prepared: bool) -> PSQLPyResult<Statement> {
        if prepared {
            return Ok(self.connection.prepare_cached(query).await?);
        }

        let prepared = self.connection.prepare(query).await?;
        self.drop_prepared(&prepared).await?;
        Ok(prepared)
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
    #[allow(clippy::used_underscore_items)]
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
    #[allow(clippy::used_underscore_items)]
    async fn commit(&mut self) -> PSQLPyResult<()> {
        self._commit().await?;
        self.in_transaction = false;

        Ok(())
    }

    #[allow(clippy::used_underscore_items)]
    async fn rollback(&mut self) -> PSQLPyResult<()> {
        self._rollback().await?;
        self.in_transaction = false;

        Ok(())
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
    #[must_use]
    pub fn in_transaction(&self) -> bool {
        match self {
            PSQLPyConnection::PoolConn(conn) => conn.in_transaction,
            PSQLPyConnection::SingleConnection(conn) => conn.in_transaction,
        }
    }

    #[must_use]
    pub fn copy_type_cache(&self) -> &CopyTypeCache {
        match self {
            PSQLPyConnection::PoolConn(conn) => &conn.copy_type_cache,
            PSQLPyConnection::SingleConnection(conn) => &conn.copy_type_cache,
        }
    }

    /// Prepare internal `PSQLPy` statement
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    pub async fn prepare_statement(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
    ) -> PSQLPyResult<PsqlpyStatement> {
        StatementBuilder::new(&querystring, &parameters, self, Some(true))
            .build()
            .await
    }

    /// Execute prepared `PSQLPy` statement.
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    pub async fn execute_statement(
        &self,
        statement: &PsqlpyStatement,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let result = self
            .query(statement.statement_query()?, &statement.params())
            .await?;

        Ok(PSQLDriverPyQueryResult::new(result))
    }

    /// Execute raw querystring without parameters.
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    pub async fn execute_no_params(
        &self,
        querystring: String,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let prepared = prepared.unwrap_or(true);
        let result = if prepared {
            self.query(&querystring, &[]).await
        } else {
            self.query_typed(&querystring, &[]).await
        };

        let return_result = result.map_err(|err| {
            RustPSQLDriverError::ConnectionExecuteError(format!(
                "Cannot execute query, error - {err}"
            ))
        })?;

        Ok(PSQLDriverPyQueryResult::new(return_result))
    }

    /// Execute raw query with parameters.
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
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
        let result = if prepared {
            self.query(statement.statement_query()?, &statement.params())
                .await
        } else {
            self.query_typed(statement.raw_query(), &statement.params_typed())
                .await
        };

        let return_result = result.map_err(|err| {
            RustPSQLDriverError::ConnectionExecuteError(format!(
                "Cannot execute query, error - {err}"
            ))
        })?;

        Ok(PSQLDriverPyQueryResult::new(return_result))
    }

    /// Execute many queries without return.
    ///
    /// ## Performance model
    ///
    /// Two coupled mechanisms give this method its throughput:
    ///
    /// 1. **Pipelining.** All `Bind`/`Execute` messages are issued against the
    ///    same connection via concurrently-polled futures (`FuturesOrdered`).
    ///    tokio-postgres dispatches them back-to-back without waiting for
    ///    intermediate replies, eliminating the per-row round-trip stall that
    ///    a naive `for ... await` loop produces.
    ///
    /// 2. **Single transactional fsync.** Every standalone `INSERT`/`UPDATE`/
    ///    `DELETE` outside a transaction is its own implicit auto-commit, and
    ///    `PostgreSQL` fsyncs the WAL per commit. Pipelining alone collapses
    ///    network latency but leaves N fsyncs on the table, capping throughput
    ///    well below what a "real" batch achieves. Wrapping the pipelined
    ///    batch in a single transaction reduces this to one fsync.
    ///
    /// Locally-measured: 1000-row INSERT batch went from ~1300 ms sequential
    /// → ~1000 ms pipelined alone → ~32 ms pipelined within a transaction.
    /// The transaction wrap is what produces the order-of-magnitude win;
    /// pipelining alone is insufficient. This matches asyncpg's `executemany`
    /// behaviour, which the project benchmarks against (see issue #167).
    ///
    /// ## Transaction wrapping policy
    ///
    /// When the caller is **not** already in a transaction (the connection's
    /// `in_transaction()` flag is `false`), the batch is wrapped in an
    /// implicit `BEGIN`/`COMMIT`. On error, `ROLLBACK` is issued before
    /// returning, leaving the connection in a clean state.
    ///
    /// When the caller **is** already in a transaction (this is invoked via
    /// `Transaction::execute_many`), the batch is wrapped in a SAVEPOINT
    /// (`SAVEPOINT psqlpy_execute_many` … `RELEASE` on success;
    /// `ROLLBACK TO` + `RELEASE` on failure). The savepoint costs two extra
    /// pipelineable round-trips but makes the failure contract symmetric
    /// across both call sites: a failed batch never poisons the caller's
    /// surrounding transaction. Without the savepoint, a single failing row
    /// would leave the outer transaction in aborted state and force the
    /// caller to roll back work they may have wanted to keep — a footgun
    /// that's hard to document away when the same method name behaves
    /// differently on a `Connection` vs a `Transaction`.
    ///
    /// asyncpg does not auto-savepoint; we deliberately diverge here. The
    /// reasoning is that psqlpy's `Connection::execute_many` *must* wrap in
    /// a transaction to get the fsync win, so the failure-isolation
    /// asymmetry between the two call sites already exists — savepoints
    /// just bring `Transaction::execute_many` into line.
    ///
    /// ## Breaking change vs prior implementation (0.12.0)
    ///
    /// **`Connection::execute_many`**: previously each row was an independent
    /// auto-commit, so a mid-batch failure left earlier rows committed. The
    /// new `BEGIN`/`COMMIT` wrap makes the whole batch atomic.
    ///
    /// **`Transaction::execute_many`**: previously a batch failure left the
    /// outer transaction in an aborted state, requiring the caller to issue
    /// `ROLLBACK`. Now the batch is wrapped in `SAVEPOINT psqlpy_execute_many`;
    /// on failure the savepoint is rolled back and the outer transaction
    /// remains live. **Callers that catch the error and explicitly call
    /// `transaction.rollback()` should be updated to omit that call** —
    /// the outer transaction is still usable after a batch failure.
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    pub async fn execute_many(
        &self,
        querystring: String,
        parameters: Option<Vec<pyo3::Py<PyAny>>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<()> {
        let Some(parameters) = parameters else {
            return Ok(());
        };
        if parameters.is_empty() {
            return Ok(());
        }

        let prepared = prepared.unwrap_or(true);

        // Build statement once using the first param set to resolve types and
        // (for the prepared path) obtain the server-side Statement handle.
        let template = StatementBuilder::new(
            &querystring,
            &Some(parameters[0].clone()),
            self,
            Some(prepared),
        )
        .build()
        .await
        .map_err(|err| {
            RustPSQLDriverError::ConnectionExecuteError(format!(
                "Cannot build statement in execute_many: {err}"
            ))
        })?;

        let prepared_stmt: Option<Statement> = if prepared {
            Some(template.statement_query()?.clone())
        } else {
            None
        };
        let param_types: Vec<Type> = template.param_types().to_vec();
        let raw_query = template.raw_query().to_string();
        // Named-parameter names are already computed inside StatementBuilder::build().
        let param_names: Option<Vec<String>> = template.param_names().map(<[_]>::to_vec);

        // Two GIL passes total: one inside StatementBuilder::build() for row 0,
        // one here for all remaining rows — independent of batch size, not per-row.
        let first_pp = template.into_prepared_parameters();
        let remaining_pp: PSQLPyResult<Vec<_>> = if parameters.len() > 1 {
            Python::with_gil(|gil| {
                parameters[1..]
                    .iter()
                    .map(|param_set| {
                        ParametersBuilder::new(Some(param_set), Some(param_types.clone()), vec![])
                            .prepare_with_gil(gil, param_names.clone())
                    })
                    .collect()
            })
        } else {
            Ok(vec![])
        };

        let mut all_pp = vec![first_pp];
        all_pp.extend(remaining_pp?);

        let wrap = if self.in_transaction() {
            ExecuteManyWrap::Savepoint
        } else {
            ExecuteManyWrap::Transaction
        };

        self.batch_execute(wrap.open_sql()).await.map_err(|err| {
            RustPSQLDriverError::ConnectionExecuteError(format!(
                "Cannot open transaction wrap in execute_many: {err}"
            ))
        })?;

        let batch_result = self
            .run_pipelined_batch(prepared_stmt.as_ref(), &raw_query, &all_pp, prepared)
            .await;

        let close_sql = wrap.close_sql(batch_result.is_ok());
        let close_result = self.batch_execute(close_sql).await;

        match (batch_result, close_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(close_err)) => Err(RustPSQLDriverError::ConnectionExecuteError(format!(
                "Failed to finalize execute_many wrap: {close_err}"
            ))),
            // When the batch already failed, the close path is best-effort:
            // the original error is the root cause and carries the diagnostic
            // the caller needs. A failed ROLLBACK / ROLLBACK TO is almost
            // always a downstream consequence of the same connection issue.
            (Err(batch_err), _) => Err(batch_err),
        }
    }

    /// Pipeline the bound parameter sets across a single connection.
    ///
    /// All futures are pushed into a `FuturesOrdered` and polled together so
    /// tokio-postgres can issue their `Bind`/`Execute` messages back-to-back.
    /// On the first error we *keep draining* remaining futures (rather than
    /// short-circuiting with `?`) so already-sent messages can be acknowledged
    /// and the connection returns to a quiescent state before the caller
    /// issues the close-wrap statement. The first error is what we propagate.
    ///
    /// The `prepared_stmt` is already resolved by `execute_many` — no second
    /// prepare call is issued here.
    ///
    /// # TODO(bind-execute-many)
    ///
    /// The per-row `FuturesOrdered<Client::query>` loop below is the layered
    /// fallback for batched execution. It should be replaced by a future
    /// `tokio_postgres::Client::bind_execute_many(&Statement, impl Iterator<Item = Params>)`
    /// primitive once it lands upstream in sfackler/rust-postgres.
    ///
    /// Target behaviour: pack Bind+Execute frames for all rows into a shared
    /// `BytesMut` (constants: `_EXECUTE_MANY_BUF_NUM=4`, `_EXECUTE_MANY_BUF_SIZE=32768`),
    /// issue a single trailing Sync, and writev ~128 KiB per round-trip.
    /// Reference algorithm: asyncpg `coreproto.pyx:1022-1092`.
    ///
    /// Tracking: no upstream issue open yet in sfackler/rust-postgres —
    /// file one at <https://github.com/sfackler/rust-postgres/issues> if the
    /// primitive is not yet tracked.
    #[allow(clippy::type_complexity)]
    async fn run_pipelined_batch(
        &self,
        prepared_stmt: Option<&Statement>,
        raw_query: &str,
        all_params: &[crate::statement::parameters::PreparedParameters],
        prepared: bool,
    ) -> PSQLPyResult<()> {
        if prepared {
            let stmt = prepared_stmt.expect("prepared_stmt required when prepared=true");

            let param_boxes: Vec<Box<[&(dyn ToSql + Sync)]>> =
                all_params.iter().map(|p| p.params()).collect();

            let ordered: FuturesOrdered<_> =
                param_boxes.iter().map(|p| self.query(stmt, p)).collect();

            drain_ordered(ordered).await
        } else {
            let param_boxes: Vec<Box<[(&(dyn ToSql + Sync), Type)]>> =
                all_params.iter().map(|p| p.params_typed()).collect();

            let ordered: FuturesOrdered<_> = param_boxes
                .iter()
                .map(|p| self.query_typed(raw_query, p))
                .collect();

            drain_ordered(ordered).await
        }
    }

    /// Execute raw query with parameters. Return one raw row
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    /// Or if cannot build statement.
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
                    .prepare(statement.raw_query(), true)
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

        Ok(result)
    }

    /// Execute raw query with parameters. Return one row
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    /// Or if cannot build statement.
    pub async fn fetch_row(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<PSQLDriverSinglePyQueryResult> {
        let result = self
            .fetch_row_raw(querystring, parameters, prepared)
            .await?;

        Ok(PSQLDriverSinglePyQueryResult::new(result))
    }

    /// Execute raw query with parameters. Return single python object
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    /// Or if cannot build statement.
    pub async fn fetch_val(
        &self,
        querystring: String,
        parameters: Option<pyo3::Py<PyAny>>,
        prepared: Option<bool>,
    ) -> PSQLPyResult<pyo3::Py<PyAny>> {
        let result = self
            .fetch_row_raw(querystring, parameters, prepared)
            .await?;

        Python::with_gil(|gil| match result.columns().first() {
            Some(first_column) => postgres_to_py(gil, &result, first_column, 0, &None),
            None => Ok(gil.None()),
        })
    }

    /// Create new sink for COPY operation.
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    /// Or if cannot build statement.
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

    /// Create and open new transaction.
    ///
    /// Unsafe here isn't a problem cuz it is stored within
    /// the struct with the connection created this transaction.
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    /// Or if cannot build statement.
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

    /// Create new Portal (server-side byte cursor).
    ///
    /// # Errors
    /// May return error if there is some problem with DB communication.
    /// Or if cannot build statement.
    pub async fn portal(
        &mut self,
        querystring: Option<&String>,
        parameters: &Option<pyo3::Py<PyAny>>,
        statement: Option<&PsqlpyStatement>,
    ) -> PSQLPyResult<(PSQLPyTransaction, tp_Portal)> {
        let stmt = if let Some(stmt) = statement {
            stmt
        } else {
            let Some(querystring) = querystring else {
                return Err(RustPSQLDriverError::ConnectionExecuteError(
                    "Can't create cursor without querystring".into(),
                ));
            };

            &StatementBuilder::new(querystring, parameters, self, Some(false))
                .build()
                .await?
        };

        let transaction = self.transaction().await?;
        let inner_portal = transaction.portal(stmt.raw_query(), &stmt.params()).await?;

        Ok((transaction, inner_portal))
    }
}

/// How `execute_many` brackets its pipelined batch.
///
/// The variant is chosen at call-time from `PSQLPyConnection::in_transaction()`:
/// a connection that is not already in a transaction gets the implicit
/// `BEGIN`/`COMMIT`; one that is already inside a transaction uses a savepoint
/// so failure of the batch can never poison the caller's surrounding work.
///
/// The savepoint name (`psqlpy_execute_many`) is internal — it collides only
/// with a user-managed savepoint of the same name, which would require the
/// caller to be reaching past the public API.
#[derive(Clone, Copy)]
enum ExecuteManyWrap {
    Transaction,
    Savepoint,
}

impl ExecuteManyWrap {
    fn open_sql(self) -> &'static str {
        match self {
            ExecuteManyWrap::Transaction => "BEGIN",
            ExecuteManyWrap::Savepoint => "SAVEPOINT psqlpy_execute_many",
        }
    }

    fn close_sql(self, batch_ok: bool) -> &'static str {
        match (self, batch_ok) {
            (ExecuteManyWrap::Transaction, true) => "COMMIT",
            (ExecuteManyWrap::Transaction, false) => "ROLLBACK",
            (ExecuteManyWrap::Savepoint, true) => "RELEASE SAVEPOINT psqlpy_execute_many",
            (ExecuteManyWrap::Savepoint, false) => {
                "ROLLBACK TO SAVEPOINT psqlpy_execute_many; RELEASE SAVEPOINT psqlpy_execute_many"
            }
        }
    }
}
