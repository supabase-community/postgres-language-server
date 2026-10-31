use std::sync::Arc;

use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use crate::workspace::StatementId;

/// The tree-sitter trees of the statements of a document. They are kept until the statement is
/// no longer in the document, see [`TreeSitterStore::retain`].
pub struct TreeSitterStore {
    db: Mutex<FxHashMap<StatementId, Arc<tree_sitter::Tree>>>,
    parser: Mutex<tree_sitter::Parser>,
}

impl TreeSitterStore {
    pub fn new() -> TreeSitterStore {
        let mut parser = tree_sitter::Parser::new();
        #[expect(
            clippy::expect_used,
            reason = "only fails if the grammar was generated for an incompatible tree-sitter version, which the tests catch"
        )]
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
        self.db.lock().len()
    }

    /// Drops every cached tree, and the memory of the map.
    pub fn clear(&self) {
        drop(std::mem::take(&mut *self.db.lock()));
    }

    /// Drops the trees of the statements for which `keep` returns `false`.
    pub fn retain(&self, keep: impl Fn(&StatementId) -> bool) {
        super::parse::retain(&mut self.db.lock(), keep);
    }

    /// The number of trees that the map has room for.
    #[cfg(test)]
    pub fn capacity(&self) -> usize {
        self.db.lock().capacity()
    }

    pub fn get_or_cache_tree(&self, statement: &StatementId) -> Arc<tree_sitter::Tree> {
        // First check cache
        {
            let cache = self.db.lock();
            if let Some(existing) = cache.get(statement) {
                return existing.clone();
            }
        }

        // Cache miss - parse outside of cache lock to avoid deadlock
        let mut parser = self.parser.lock();
        #[expect(
            clippy::expect_used,
            reason = "parsing only fails without a language, which `new` sets, or on a timeout or cancellation, which are never set"
        )]
        let tree = Arc::new(
            parser
                .parse(statement.content(), None)
                .expect("the parser has a language"),
        );
        drop(parser);

        // Insert into cache
        {
            let mut cache = self.db.lock();
            // Double-check in case another thread inserted while we were parsing
            if let Some(existing) = cache.get(statement) {
                return existing.clone();
            }
            cache.insert(statement.clone(), tree.clone());
        }

        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_trees_releases_the_map() {
        let store = TreeSitterStore::new();
        let statements: Vec<StatementId> = (0..200)
            .map(|i| StatementId::new(&format!("select {i};")))
            .collect();
        let fill = || {
            for id in &statements {
                store.get_or_cache_tree(id);
            }
        };

        fill();
        store.retain(|id| *id == statements[0]);
        assert_eq!(store.len(), 1);
        assert!(store.capacity() < 64, "{}", store.capacity());

        fill();
        store.clear();
        assert_eq!(store.capacity(), 0);
    }
}
