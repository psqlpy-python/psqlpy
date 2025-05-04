use std::collections::HashMap;

use once_cell::sync::Lazy;
use postgres_types::Type;
use tokio::sync::RwLock;
use tokio_postgres::Statement;

use super::{query::QueryString, utils::hash_str};

#[derive(Default)]
pub(crate) struct StatementsCache(HashMap<u64, StatementCacheInfo>);

impl StatementsCache {
    pub fn add_cache(&mut self, query: &QueryString, inner_stmt: &Statement) {
        self.0
            .insert(query.hash(), StatementCacheInfo::new(query, inner_stmt));
    }

    pub fn get_cache(&self, querystring: &String) -> Option<StatementCacheInfo> {
        let qs_hash = hash_str(&querystring);

        if let Some(cache_info) = self.0.get(&qs_hash) {
            return Some(cache_info.clone());
        }

        None
    }
}

#[derive(Clone)]
pub(crate) struct StatementCacheInfo {
    pub(crate) query: QueryString,
    pub(crate) inner_stmt: Statement,
}

impl StatementCacheInfo {
    fn new(query: &QueryString, inner_stmt: &Statement) -> Self {
        return Self {
            query: query.clone(),
            inner_stmt: inner_stmt.clone(),
        };
    }

    pub(crate) fn types(&self) -> Vec<Type> {
        self.inner_stmt.params().to_vec()
    }
}

pub(crate) static STMTS_CACHE: Lazy<RwLock<StatementsCache>> =
    Lazy::new(|| RwLock::new(Default::default()));
