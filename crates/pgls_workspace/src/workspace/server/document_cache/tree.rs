use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use crate::workspace::StatementId;

/// The tree-sitter trees of the statements of a document. They are kept until the statement is
/// no longer in the document, see [`TreeSitterStore::remove`].
pub struct TreeSitterStore {
    db: Mutex<FxHashMap<StatementId, Arc<tree_sitter::Tree>>>,
    parser: Mutex<tree_sitter::Parser>,
}

impl TreeSitterStore {
    pub fn new() -> TreeSitterStore {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");

        TreeSitterStore {
            db: Mutex::default(),
            parser: Mutex::new(parser),
        }
    }

    /// The number of statements with a cached tree.
    pub fn len(&self) -> usize {
        self.db.lock().unwrap().len()
    }

    /// Drops every cached tree.
    pub fn clear(&self) {
        self.db.lock().unwrap().clear();
    }

    /// Drops the tree of `statement`.
    pub fn remove(&self, statement: &StatementId) {
        self.db.lock().unwrap().remove(statement);
    }

    /// The tree of `statement`. It is only cached if `may_insert` returns `true`, see
    /// [`PgQueryStore::get_or_cache_ast`](super::parse::PgQueryStore::get_or_cache_ast).
    pub fn get_or_cache_tree(
        &self,
        statement: &StatementId,
        may_insert: impl Fn() -> bool,
    ) -> Arc<tree_sitter::Tree> {
        // First check cache
        {
            let cache = self.db.lock().unwrap();
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
            if may_insert() {
                cache.insert(statement.clone(), tree.clone());
            }
        }

        tree
    }
}
