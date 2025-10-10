use crate::{exceptions::rust_errors::PSQLPyResult, query_result::PSQLDriverPyQueryResult};

use super::structs::PSQLPyTransaction;
use tokio_postgres::{Portal as tp_Portal, ToStatement};

impl PSQLPyTransaction {
    /// Query the portal (server-side cursor) to retrieve next elements.
    ///
    /// # Errors
    /// May return error if there is a problem with DB communication.
    pub async fn query_portal(
        &self,
        portal: &tp_Portal,
        size: i32,
    ) -> PSQLPyResult<PSQLDriverPyQueryResult> {
        let portal_res = match self {
            PSQLPyTransaction::PoolTransaction(txid) => txid.query_portal(portal, size).await?,
            PSQLPyTransaction::SingleTransaction(txid) => txid.query_portal(portal, size).await?,
        };

        Ok(PSQLDriverPyQueryResult::new(portal_res))
    }

    /// Create new portal (server-side cursor).
    ///
    /// # Errors
    /// May return error if there is a problem with DB communication.
    pub async fn portal<T>(
        &self,
        querystring: &T,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> PSQLPyResult<tp_Portal>
    where
        T: ?Sized + ToStatement,
    {
        let portal: tp_Portal = match self {
            PSQLPyTransaction::PoolTransaction(conn) => conn.bind(querystring, params).await?,
            PSQLPyTransaction::SingleTransaction(conn) => conn.bind(querystring, params).await?,
        };

        Ok(portal)
    }
}
