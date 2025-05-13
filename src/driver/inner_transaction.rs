use deadpool_postgres::Transaction as dp_Transaction;
use postgres_types::ToSql;
use tokio_postgres::{Portal, Row, ToStatement, Transaction as tp_Transaction};

use crate::exceptions::rust_errors::PSQLPyResult;

pub enum PsqlpyTransaction {
    PoolTrans(dp_Transaction<'static>),
    SingleConnTrans(tp_Transaction<'static>),
}

impl PsqlpyTransaction {
    async fn commit(self) -> PSQLPyResult<()> {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => Ok(p_txid.commit().await?),
            PsqlpyTransaction::SingleConnTrans(s_txid) => Ok(s_txid.commit().await?),
        }
    }

    async fn rollback(self) -> PSQLPyResult<()> {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => Ok(p_txid.rollback().await?),
            PsqlpyTransaction::SingleConnTrans(s_txid) => Ok(s_txid.rollback().await?),
        }
    }

    async fn savepoint(&mut self, sp_name: &str) -> PSQLPyResult<()> {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => {
                p_txid.savepoint(sp_name).await?;
                Ok(())
            }
            PsqlpyTransaction::SingleConnTrans(s_txid) => {
                s_txid.savepoint(sp_name).await?;
                Ok(())
            }
        }
    }

    async fn release_savepoint(&self, sp_name: &str) -> PSQLPyResult<()> {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => {
                p_txid
                    .batch_execute(format!("RELEASE SAVEPOINT {sp_name}").as_str())
                    .await?;
                Ok(())
            }
            PsqlpyTransaction::SingleConnTrans(s_txid) => {
                s_txid
                    .batch_execute(format!("RELEASE SAVEPOINT {sp_name}").as_str())
                    .await?;
                Ok(())
            }
        }
    }

    async fn rollback_savepoint(&self, sp_name: &str) -> PSQLPyResult<()> {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => {
                p_txid
                    .batch_execute(format!("ROLLBACK TO SAVEPOINT {sp_name}").as_str())
                    .await?;
                Ok(())
            }
            PsqlpyTransaction::SingleConnTrans(s_txid) => {
                s_txid
                    .batch_execute(format!("ROLLBACK TO SAVEPOINT {sp_name}").as_str())
                    .await?;
                Ok(())
            }
        }
    }

    async fn bind<T>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> PSQLPyResult<Portal>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => Ok(p_txid.bind(statement, params).await?),
            PsqlpyTransaction::SingleConnTrans(s_txid) => {
                Ok(s_txid.bind(statement, params).await?)
            }
        }
    }

    pub async fn query_portal(&self, portal: &Portal, size: i32) -> PSQLPyResult<Vec<Row>> {
        match self {
            PsqlpyTransaction::PoolTrans(p_txid) => Ok(p_txid.query_portal(portal, size).await?),
            PsqlpyTransaction::SingleConnTrans(s_txid) => {
                Ok(s_txid.query_portal(portal, size).await?)
            }
        }
    }
}
