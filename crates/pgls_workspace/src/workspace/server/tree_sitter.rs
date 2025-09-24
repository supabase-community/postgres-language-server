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
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");

        TreeSitterStore {
            db: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap(),
            )),
            parser: Mutex::new(parser),
        }
    }

    pub fn get_or_cache_tree(&self, statement: &StatementId) -> Arc<tree_sitter::Tree> {
        // First check cache
        {
            let mut cache = self.db.lock().unwrap();
            if let Some(existing) = cache.get(statement) {
                return existing.clone();
            }
        }

        // Cache miss - parse outside of cache lock to avoid deadlock
        let mut parser = self.parser.lock().unwrap();
        let tree = Arc::new(parser.parse(statement.content(), None).unwrap());
        drop(parser);

        // Insert into cache
        {
            let mut cache = self.db.lock().unwrap();
            // Double-check in case another thread inserted while we were parsing
            if let Some(existing) = cache.get(statement) {
                return existing.clone();
            }
            cache.put(statement.clone(), tree.clone());
        }

        tree
    }
}
