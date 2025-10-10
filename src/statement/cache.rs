use std::collections::HashMap;

use postgres_types::Type;
use tokio::sync::RwLock;
use tokio_postgres::Statement;

use super::{parameters::Column, query::QueryString, utils::hash_str};

#[derive(Default)]
pub(crate) struct StatementsCache(HashMap<u64, StatementCacheInfo>);

impl StatementsCache {
    pub fn add_cache(&mut self, query: &QueryString, inner_stmt: &Statement) {
        self.0
            .insert(query.hash(), StatementCacheInfo::new(query, inner_stmt));
    }

    pub fn get_cache(&self, querystring: &String) -> Option<StatementCacheInfo> {
        let qs_hash = hash_str(querystring);

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
        Self {
            query: query.clone(),
            inner_stmt: inner_stmt.clone(),
        }
    }

    pub(crate) fn types(&self) -> Vec<Type> {
        self.inner_stmt.params().to_vec()
    }

    pub(crate) fn columns(&self) -> Vec<Column> {
        self.inner_stmt
            .columns()
            .iter()
            .map(|column| Column::new(column.name().to_string(), column.table_oid()))
            .collect::<Vec<Column>>()
    }
}

pub(crate) static STMTS_CACHE: std::sync::LazyLock<RwLock<StatementsCache>> =
    std::sync::LazyLock::new(|| RwLock::new(StatementsCache::default()));
