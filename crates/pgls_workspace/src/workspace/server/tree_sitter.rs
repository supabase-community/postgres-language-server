use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use super::statement_identifier::StatementId;

/// The tree-sitter trees of the statements of a document. They are kept until the statement is
/// no longer in the document, see [`TreeSitterStore::retain`].
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

    /// Drops the trees of the statements for which `keep` returns `false`.
    pub fn retain(&self, keep: impl Fn(&StatementId) -> bool) {
        self.db.lock().unwrap().retain(|id, _| keep(id));
    }

    pub fn get_or_cache_tree(&self, statement: &StatementId) -> Arc<tree_sitter::Tree> {
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
            cache.insert(statement.clone(), tree.clone());
        }

        tree
    }
}
