use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use pgt_query_ext::diagnostics::*;

use super::statement_identifier::StatementId;

const DEFAULT_CACHE_SIZE: usize = 1000;

pub struct PgQueryStore {
    db: Mutex<LruCache<StatementId, Arc<Result<pgt_query_ext::NodeEnum, SyntaxDiagnostic>>>>,
}

impl PgQueryStore {
    pub fn new() -> PgQueryStore {
        PgQueryStore {
            db: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap(),
            )),
        }
    }

    pub fn get_or_cache_ast(
        &self,
        statement: &StatementId,
    ) -> Arc<Result<pgt_query_ext::NodeEnum, SyntaxDiagnostic>> {
        let mut cache = self.db.lock().unwrap();

        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        let r = Arc::new(pgt_query_ext::parse(statement.content()).map_err(SyntaxDiagnostic::from));
        cache.put(statement.clone(), r.clone());
        r
    }
}
