use std::sync::Arc;

use tokio_postgres::{Client, Config};

use crate::driver::psqlpy_manager::PsqlpyClient;

#[derive(Debug)]
pub struct PoolConnection {
    pub connection: PsqlpyClient,
    pub in_transaction: bool,
    pub in_cursor: bool,
    pub pg_config: Arc<Config>,
}

impl PoolConnection {
    #[must_use]
    pub fn new(connection: PsqlpyClient, pg_config: Arc<Config>) -> Self {
        Self {
            connection,
            in_transaction: false,
            in_cursor: false,
            pg_config,
        }
    }
}

#[derive(Debug)]
pub struct SingleConnection {
    pub connection: Client,
    pub in_transaction: bool,
    pub in_cursor: bool,
    pub pg_config: Arc<Config>,
}

impl SingleConnection {
    #[must_use]
    pub fn new(connection: Client, pg_config: Arc<Config>) -> Self {
        Self {
            connection,
            in_transaction: false,
            in_cursor: false,
            pg_config,
        }
    }
}

#[derive(Debug)]
pub enum PSQLPyConnection {
    PoolConn(PoolConnection),
    SingleConnection(SingleConnection),
}
