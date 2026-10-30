//! Everything that is cached for a document.
//!
//! - Per statement text: the parse results and the tree-sitter trees. An edit only drops what
//!   was cached for the statements it changed.
//! - Per [`CacheKey`]: what also depends on the settings and the database, like the analysis of
//!   the document. It is dropped when the key changes, and the analysis when the content does.
//!
//! All of it can be recomputed. [`DocumentCache::usage`] says how much a document keeps, and
//! the workspace drops the caches of the least recently used documents when all of them keep more
//! than [`DOCUMENT_CACHE_ENTRIES_MAX`], see [`CacheBudget`].

#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
use pgls_catalog::CatalogBase;
use pgls_query_ext::{diagnostics::SyntaxDiagnostic, function_body::EmbeddedStatement};

use rustc_hash::FxHashSet;

use super::{StatementId, analysis::DocumentAnalysis};
use crate::workspace::ProjectKey;

mod budget;
mod parse;
mod tree;
mod walk;

pub(crate) use budget::CacheBudget;
pub(crate) use parse::ParseResult;
use parse::PgQueryStore;
use tree::TreeSitterStore;
pub(crate) use walk::{ContentChange, ResolvedStatement, WalkCache};

/// The number of cache entries that all open documents keep together, beyond the document in
/// use, see [`DocumentCache::usage`]. When they keep more, the caches of the least recently used
/// documents are dropped, and recomputed when they are used again.
///
/// An entry is a statement's parse result or tree, or the references of a statement, and takes
/// about 10 KB on average (measured on migration files with tables, indexes, SQL functions and
/// queries), so this is about 200 MB. That keeps a few files of several thousand statements
/// fully cached, so that switching between them doesn't parse them again. Without a limit, 20
/// open files of 5,000 statements each kept 2.7 GB.
pub const DOCUMENT_CACHE_ENTRIES_MAX: usize = 20_000;

/// What a walk checkpoint counts as in [`DocumentCache::usage`]. It copies the maps of the
/// objects that the file changed before it, about 100 bytes per object, so a checkpoint after
/// the 5,000 tables of a 20,000-statement file takes about as much as 50 entries.
const CHECKPOINT_ENTRIES: usize = 50;

/// What the caches of an open document hold, for tests of their memory.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentCacheStats {
    /// The distinct statements of the document, including the bodies of SQL functions. The
    /// parse results and trees are keyed by the text of the statement.
    pub statements: usize,
    pub parse_results: usize,
    pub trees: usize,
    pub analysis: bool,
    /// The statements whose references are kept, see [`WalkCache`].
    pub resolved_statements: usize,
    pub walk_checkpoints: usize,
}

/// What an edit invalidated in a document's cache. On a long file, the analysis, the copies of
/// the catalog in the walk checkpoints and the snapshot hold every object the file defines, and
/// dropping them takes tens of milliseconds, so [`StaleEntries::drop_in_background`] drops them
/// off the path of the edit.
#[must_use]
pub(crate) struct StaleEntries {
    analysis: Option<Arc<DocumentAnalysis>>,
    walk: WalkCache,
}

/// The stack of the thread that drops [`StaleEntries`]. Dropping a statement's tree is recursive
/// like parsing it, so this is the stack that the CLI gives its workers for parsing.
#[cfg(not(target_family = "wasm"))]
const DROP_THREAD_STACK_BYTES: usize = 8 * 1024 * 1024;

impl StaleEntries {
    /// Drops the entries on a thread of its own, so that the edit doesn't wait for it. If the
    /// thread can't be started, and in WebAssembly, which has no threads, they are dropped right
    /// away.
    pub(crate) fn drop_in_background(self) {
        if self.analysis.is_none() && self.walk.is_empty() {
            return;
        }
        #[cfg(target_family = "wasm")]
        drop(self);
        #[cfg(not(target_family = "wasm"))]
        self.drop_on_thread();
    }

    #[cfg(not(target_family = "wasm"))]
    fn drop_on_thread(self) {
        let spawned = std::thread::Builder::new()
            .name("pgls-cache-drop".into())
            .stack_size(DROP_THREAD_STACK_BYTES)
            .spawn(move || drop(self));
        if let Err(error) = spawned {
            tracing::debug!("Dropped the stale cache entries of an edit in place: {error}");
        }
    }
}

/// What the caches of a document depend on besides its text.
#[derive(Clone)]
pub(crate) struct CacheKey {
    /// Changes whenever the settings change.
    settings_version: u64,
    /// The project whose settings apply. A file can be opened by several projects, e.g. when
    /// workspace folders are nested.
    project_key: ProjectKey,
    /// The indexed database snapshot. It is built from the snapshot, so its identity is also
    /// that of the snapshot. `None` without a database.
    catalog_base: Option<Arc<CatalogBase>>,
}

impl CacheKey {
    pub(crate) fn new(
        settings_version: u64,
        project_key: ProjectKey,
        catalog_base: Option<Arc<CatalogBase>>,
    ) -> Self {
        Self {
            settings_version,
            project_key,
            catalog_base,
        }
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.settings_version == other.settings_version
            && self.project_key == other.project_key
            && match (&self.catalog_base, &other.catalog_base) {
                (Some(this), Some(other)) => Arc::ptr_eq(this, other),
                (None, None) => true,
                _ => false,
            }
    }
}

/// The caches of a document.
pub(crate) struct DocumentCache {
    parses: PgQueryStore,
    trees: TreeSitterStore,
    keyed: Mutex<KeyedCaches>,
}

/// The caches that are only valid for one [`CacheKey`].
#[derive(Default)]
struct KeyedCaches {
    key: Option<CacheKey>,
    /// The generation of the document's content they are for, see [`Document::generation`].
    /// A request on a previous generation, which still runs while the document changed, neither
    /// reads nor writes them.
    ///
    /// [`Document::generation`]: super::document::Document::generation
    generation: u64,
    analysis: Option<Arc<DocumentAnalysis>>,
    walk: WalkCache,
}

impl DocumentCache {
    pub(crate) fn new() -> Self {
        Self {
            parses: PgQueryStore::new(),
            trees: TreeSitterStore::new(),
            keyed: Mutex::default(),
        }
    }

    /// The analysis of the content of `generation` for `key`. `analyse` computes it if it isn't
    /// cached. It returns `None` if the analysis stopped early, which isn't cached.
    pub(crate) fn analysis(
        &self,
        key: &CacheKey,
        generation: u64,
        analyse: impl FnOnce() -> Option<DocumentAnalysis>,
    ) -> Option<Arc<DocumentAnalysis>> {
        // Held while analysing, so that concurrent requests wait for one analysis.
        let Some(mut keyed) = self.keyed(key, generation) else {
            return analyse().map(Arc::new);
        };
        if let Some(analysis) = &keyed.analysis {
            return Some(Arc::clone(analysis));
        }
        let analysis = Arc::new(analyse()?);
        keyed.analysis = Some(Arc::clone(&analysis));
        Some(analysis)
    }

    /// How much the document keeps, in entries: one per parse result, tree and resolved
    /// statement, see [`DOCUMENT_CACHE_ENTRIES_MAX`], and [`CHECKPOINT_ENTRIES`] per walk
    /// checkpoint.
    pub(crate) fn usage(&self) -> usize {
        let (resolved, checkpoints) = self.keyed.lock().walk.len();
        self.parses.len() + self.trees.len() + resolved + checkpoints * CHECKPOINT_ENTRIES
    }

    /// Drops everything that is cached. It is recomputed when it is needed again.
    pub(crate) fn clear(&self) {
        self.parses.clear();
        self.trees.clear();
        let mut keyed = self.keyed.lock();
        *keyed = KeyedCaches {
            generation: keyed.generation,
            ..KeyedCaches::default()
        };
    }

    /// What the caches hold for a document with the top-level statements `statements`. The
    /// statements of a function body are counted if they are cached.
    pub(crate) fn stats(&self, statements: &FxHashSet<StatementId>) -> DocumentCacheStats {
        let embedded: usize = statements
            .iter()
            .filter_map(|id| self.parses.cached_embedded_len(id))
            .sum();
        let statements = statements.len() + embedded;
        let keyed = self.keyed.lock();
        let (resolved_statements, walk_checkpoints) = keyed.walk.len();
        DocumentCacheStats {
            statements,
            parse_results: self.parses.len(),
            trees: self.trees.len(),
            analysis: keyed.analysis.is_some(),
            resolved_statements,
            walk_checkpoints,
        }
    }

    /// The cached analysis, whatever its key.
    #[cfg(test)]
    pub(crate) fn cached_analysis(&self) -> Option<Arc<DocumentAnalysis>> {
        self.keyed.lock().analysis.clone()
    }

    /// The caches for `key`, emptied first if they were for another key. `None` if they are for
    /// another generation of the content.
    fn keyed(&self, key: &CacheKey, generation: u64) -> Option<MutexGuard<'_, KeyedCaches>> {
        let mut keyed = self.keyed.lock();
        if keyed.generation != generation {
            return None;
        }
        if keyed.key.as_ref() != Some(key) {
            *keyed = KeyedCaches {
                key: Some(key.clone()),
                generation,
                ..KeyedCaches::default()
            };
        }
        Some(keyed)
    }

    /// Runs `f` on what the walks over the content of `generation` left for `key`. It is not
    /// held while walking, so that requests on the same document don't wait for each other. A
    /// request on a previous generation gets an empty cache, which is dropped.
    pub(crate) fn with_walk<R>(
        &self,
        key: &CacheKey,
        generation: u64,
        f: impl FnOnce(&mut WalkCache) -> R,
    ) -> R {
        match self.keyed(key, generation) {
            Some(mut keyed) => f(&mut keyed.walk),
            None => f(&mut WalkCache::default()),
        }
    }

    /// The document's content changed to `generation`. Takes out what `change` invalidates, see
    /// [`WalkCache::content_changed`], and the analysis. [`Self::retain_statements`] drops what
    /// was cached for the statements.
    pub(crate) fn content_changed(&self, change: ContentChange, generation: u64) -> StaleEntries {
        let mut keyed = self.keyed.lock();
        keyed.generation = generation;
        StaleEntries {
            analysis: keyed.analysis.take(),
            walk: keyed.walk.content_changed(change),
        }
    }

    /// The tree of the statement `id`, or why it doesn't parse.
    pub(crate) fn parse(&self, id: &StatementId) -> Arc<ParseResult> {
        self.parses.get_or_cache_ast(id)
    }

    /// The syntax error of the PL/pgSQL body of the function that `id` creates, if it creates a
    /// PL/pgSQL function.
    pub(crate) fn plpgsql_parse(&self, id: &StatementId) -> Option<Result<(), SyntaxDiagnostic>> {
        self.parses.get_or_cache_plpgsql_parse(id)
    }

    /// The statements embedded in the top-level statement `id`, see
    /// [`PgQueryStore::get_or_cache_embedded`].
    pub(crate) fn embedded(&self, id: &StatementId) -> Arc<[EmbeddedStatement]> {
        self.parses.get_or_cache_embedded(id)
    }

    /// The tree-sitter tree of the statement `id`.
    pub(crate) fn tree(&self, id: &StatementId) -> Arc<tree_sitter::Tree> {
        self.trees.get_or_cache_tree(id)
    }

    /// Drops what is cached for the statements for which `keep` returns `false`.
    pub(crate) fn retain_statements(&self, keep: impl Fn(&StatementId) -> bool) {
        self.parses.retain(&keep);
        self.trees.retain(&keep);
    }
}
