//! Everything that is cached for a document.
//!
//! - Per statement text: the parse results and the tree-sitter trees. An edit only drops what
//!   was cached for the statements it changed.
//! - Per [`CacheKey`]: what also depends on the settings and the database, like the analysis of
//!   the document. It is dropped when the key changes, and the analysis when the content does.

use std::sync::{Arc, Mutex};

use pgls_catalog::CatalogBase;
use pgls_query_ext::{diagnostics::SyntaxDiagnostic, function_body::EmbeddedStatement};

use super::{StatementId, analysis::DocumentAnalysis};
use crate::workspace::ProjectKey;

mod parse;
mod tree;
mod walk;

pub(crate) use parse::ParseResult;
use parse::PgQueryStore;
use tree::TreeSitterStore;
pub(crate) use walk::{ContentChange, ResolvedStatement, WalkCache};

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

    /// The analysis of the document for `key`. `analyse` computes it if it isn't cached.
    pub(crate) fn analysis(
        &self,
        key: &CacheKey,
        analyse: impl FnOnce() -> DocumentAnalysis,
    ) -> Arc<DocumentAnalysis> {
        // Held while analysing, so that concurrent requests wait for one analysis.
        let mut keyed = self.keyed(key);
        if let Some(analysis) = &keyed.analysis {
            return Arc::clone(analysis);
        }
        let analysis = Arc::new(analyse());
        keyed.analysis = Some(Arc::clone(&analysis));
        analysis
    }

    /// The cached analysis, whatever its key.
    #[cfg(test)]
    pub(crate) fn cached_analysis(&self) -> Option<Arc<DocumentAnalysis>> {
        self.keyed.lock().unwrap().analysis.clone()
    }

    /// The number of resolved statements and walk checkpoints, whatever their key.
    #[cfg(test)]
    pub(crate) fn walk_size(&self) -> (usize, usize) {
        self.keyed.lock().unwrap().walk.len()
    }

    /// The caches for `key`, emptied first if they were for another key.
    fn keyed(&self, key: &CacheKey) -> std::sync::MutexGuard<'_, KeyedCaches> {
        let mut keyed = self.keyed.lock().unwrap();
        if keyed.key.as_ref() != Some(key) {
            *keyed = KeyedCaches {
                key: Some(key.clone()),
                ..KeyedCaches::default()
            };
        }
        keyed
    }

    /// Runs `f` on what the walks over the document left for `key`. It is not held while
    /// walking, so that requests on the same document don't wait for each other.
    pub(crate) fn with_walk<R>(&self, key: &CacheKey, f: impl FnOnce(&mut WalkCache) -> R) -> R {
        f(&mut self.keyed(key).walk)
    }

    /// Drops what `change` invalidates, see [`WalkCache::content_changed`], and the analysis.
    /// [`Self::retain_statements`] drops what was cached for the statements.
    pub(crate) fn content_changed(&mut self, change: ContentChange) {
        let keyed = self.keyed.get_mut().unwrap();
        keyed.analysis = None;
        keyed.walk.content_changed(change);
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
