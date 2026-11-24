use std::sync::Arc;

use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use crate::workspace::StatementId;

/// The tree-sitter trees of the statements of a document. They are kept until the statement is
/// no longer in the document, see [`TreeSitterStore::remove`].
pub(super) struct TreeSitterStore {
    db: Mutex<FxHashMap<StatementId, Arc<tree_sitter::Tree>>>,
    parser: Mutex<tree_sitter::Parser>,
}

impl TreeSitterStore {
    pub(super) fn new() -> TreeSitterStore {
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
    pub(super) fn len(&self) -> usize {
        self.db.lock().len()
    }

    /// Takes every cached tree out, with the memory of the map, for the caller to drop.
    pub(super) fn take(&self) -> FxHashMap<StatementId, Arc<tree_sitter::Tree>> {
        std::mem::take(&mut *self.db.lock())
    }

    /// Drops the tree of `statement`.
    pub(super) fn remove(&self, statement: &StatementId) {
        self.db.lock().remove(statement);
    }

    /// Shrinks the map if it is less than a quarter full, see [`super::parse::shrink`].
    pub(super) fn shrink(&self) {
        super::parse::shrink(&mut self.db.lock());
    }

    /// The number of trees that the map has room for.
    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        self.db.lock().capacity()
    }

    /// The tree of `statement`. It is only cached if `may_insert` returns `true`, see
    /// [`PgQueryStore::get_or_cache_ast`](super::parse::PgQueryStore::get_or_cache_ast).
    pub(super) fn get_or_cache_tree(
        &self,
        statement: &StatementId,
        may_insert: impl Fn() -> bool,
    ) -> Arc<tree_sitter::Tree> {
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
        #[cfg(all(test, feature = "db"))]
        TreeParses::count();

        // Insert into cache
        {
            let mut cache = self.db.lock();
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

/// The statements that tree-sitter parsed on this thread, for the scale tests, see
/// [`TreeParses::during`]. The scale tests that read it need a database.
#[cfg(all(test, feature = "db"))]
pub(crate) struct TreeParses;

#[cfg(all(test, feature = "db"))]
thread_local! {
    static TREE_PARSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(all(test, feature = "db"))]
impl TreeParses {
    fn count() {
        TREE_PARSES.with(|parses| parses.set(parses.get() + 1));
    }

    /// Runs `f` and returns how many statements tree-sitter parsed meanwhile on this thread.
    pub(crate) fn during<T>(f: impl FnOnce() -> T) -> (T, usize) {
        let before = TREE_PARSES.with(std::cell::Cell::get);
        let value = f();
        (value, TREE_PARSES.with(std::cell::Cell::get) - before)
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
                store.get_or_cache_tree(id, || true);
            }
        };

        fill();
        for id in &statements[1..] {
            store.remove(id);
        }
        store.shrink();
        assert_eq!(store.len(), 1);
        assert!(store.capacity() < 64, "{}", store.capacity());

        fill();
        assert_eq!(store.take().len(), statements.len());
        assert_eq!(store.capacity(), 0);
    }
}
