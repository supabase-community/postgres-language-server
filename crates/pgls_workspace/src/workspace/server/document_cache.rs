//! Everything that is cached for a document.
//!
//! The caches of a document are keyed by the text of its statements, so that an edit only
//! drops what was cached for the statements it changed.

use std::sync::Arc;

use pgls_query_ext::{diagnostics::SyntaxDiagnostic, function_body::EmbeddedStatement};

use super::StatementId;

mod parse;
mod tree;

pub(crate) use parse::ParseResult;
use parse::PgQueryStore;
use tree::TreeSitterStore;

/// The caches of a document.
pub(crate) struct DocumentCache {
    parses: PgQueryStore,
    trees: TreeSitterStore,
}

impl DocumentCache {
    pub(crate) fn new() -> Self {
        Self {
            parses: PgQueryStore::new(),
            trees: TreeSitterStore::new(),
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
