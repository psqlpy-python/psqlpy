use std::sync::Arc;

use dashmap::DashMap;
use deadpool_postgres::Object;
use postgres_types::Type;
use tokio_postgres::{Client, Config, Statement};

/// Per-connection cache for COPY column-type introspection results.
/// Key: `(schema_name, table_name, columns_in_declaration_order)`.
/// Column order is significant: `["a","b"]` and `["b","a"]` produce different COPY targets.
/// Cache is per-checkout on `PoolConnection` — reuse the same acquired connection
/// for consecutive `copy_records_to_table` calls to benefit from this cache.
pub type CopyTypeCache = DashMap<(Option<String>, String, Vec<String>), Vec<Type>>;

#[derive(Debug)]
pub struct PoolConnection {
    pub connection: Object,
    pub in_transaction: bool,
    pub in_cursor: bool,
    pub pg_config: Arc<Config>,
    /// Per-connection cache for COPY column-type introspection results.
    pub copy_type_cache: CopyTypeCache,
}

impl PoolConnection {
    #[must_use]
    pub fn new(connection: Object, pg_config: Arc<Config>) -> Self {
        Self {
            connection,
            in_transaction: false,
            in_cursor: false,
            pg_config,
            copy_type_cache: DashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct SingleConnection {
    pub connection: Client,
    pub in_transaction: bool,
    pub in_cursor: bool,
    pub pg_config: Arc<Config>,
    /// Per-connection prepared-statement cache. Keyed by the raw query string.
    pub stmt_cache: DashMap<String, Statement>,
    /// Per-connection cache for COPY column-type introspection results.
    pub copy_type_cache: CopyTypeCache,
}

impl SingleConnection {
    #[must_use]
    pub fn new(connection: Client, pg_config: Arc<Config>) -> Self {
        Self {
            connection,
            in_transaction: false,
            in_cursor: false,
            pg_config,
            stmt_cache: DashMap::new(),
            copy_type_cache: DashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum PSQLPyConnection {
    PoolConn(PoolConnection),
    SingleConnection(SingleConnection),
}
