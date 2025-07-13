use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;

use super::statement_identifier::StatementId;

const DEFAULT_CACHE_SIZE: usize = 1000;

pub struct TreeSitterStore {
    db: Mutex<LruCache<StatementId, Arc<tree_sitter::Tree>>>,
    parser: Mutex<tree_sitter::Parser>,
}

impl TreeSitterStore {
    pub fn new() -> TreeSitterStore {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        TreeSitterStore {
            db: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap(),
            )),
            parser: Mutex::new(parser),
        }
    }

    pub fn get_or_cache_tree(&self, statement: &StatementId) -> Arc<tree_sitter::Tree> {
        let mut cache = self.db.lock().expect("Failed to lock cache");

        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        // Cache miss - drop cache lock, parse, then re-acquire to insert
        drop(cache);

        let mut parser = self.parser.lock().expect("Failed to lock parser");
        let tree = Arc::new(parser.parse(statement.content(), None).unwrap());
        drop(parser);

        let mut cache = self.db.lock().expect("Failed to lock cache");

        // Double-check after re-acquiring lock
        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        cache.put(statement.clone(), tree.clone());
        tree
    }
}
