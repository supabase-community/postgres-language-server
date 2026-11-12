//! The statements of a file, as the walk visits them.

use std::sync::Arc;

use pgls_catalog::resolve::FunctionContext;
use pgls_query_ext::function_body::{EmbeddedStatement, OffsetMap};
use pgls_text_size::{TextRange, TextSize};

#[derive(Debug)]
pub struct AnalysableStatement {
    /// Shared with the workspace's cache of parsed statements, so that analysing a file doesn't
    /// copy every statement's tree.
    pub root: Arc<pgls_query::NodeEnum>,
    pub range: TextRange,
    /// The statement text, used for precise spans of typecheck findings.
    pub sql: Option<String>,
    /// Set when the statement is the body of a SQL function. Body statements are analysed, but
    /// they don't change the catalog or the session.
    pub function: Option<FunctionContext>,
    /// Set when identifiers of the statement are client-side parameters, like psql's
    /// `:schema.table`. The parsed names are placeholders, so the statement isn't typechecked.
    pub has_identifier_parameters: bool,
    /// Where the text of the statement is in the file, when it is embedded in another statement
    /// and its text is not the source text, like a function body with escaped quotes.
    pub offsets: OffsetMap,
}

impl AnalysableStatement {
    pub fn new(root: impl Into<Arc<pgls_query::NodeEnum>>, range: TextRange) -> Self {
        Self {
            root: root.into(),
            range,
            sql: None,
            function: None,
            has_identifier_parameters: false,
            offsets: OffsetMap::default(),
        }
    }

    /// The statement `embedded`, like a statement of a SQL function body, in the statement that
    /// starts at `parent_start` in the file. `root` is the parsed `embedded.sql`.
    pub fn embedded(
        root: impl Into<Arc<pgls_query::NodeEnum>>,
        parent_start: TextSize,
        embedded: &EmbeddedStatement,
    ) -> Self {
        Self {
            offsets: embedded.offsets.clone(),
            ..Self::new(root, embedded.range + parent_start).with_sql(embedded.sql.as_str())
        }
    }

    /// The range in the file of `range` in the statement text.
    pub fn document_range(&self, range: TextRange) -> TextRange {
        self.offsets.range_to_source(range) + self.range.start()
    }

    /// The offset in the file of `offset` in the statement text.
    pub fn document_offset(&self, offset: TextSize) -> TextSize {
        self.offsets.to_source(offset) + self.range.start()
    }

    /// The offset in the statement text of `offset` in the file, which is in the statement's
    /// range.
    pub fn statement_offset(&self, offset: TextSize) -> TextSize {
        debug_assert!(self.range.contains_inclusive(offset));
        self.offsets
            .to_statement(offset.checked_sub(self.range.start()).unwrap_or_default())
    }

    pub fn with_sql(mut self, sql: impl Into<String>) -> Self {
        self.sql = Some(sql.into());
        self
    }

    pub fn with_function(mut self, function: Option<FunctionContext>) -> Self {
        self.function = function;
        self
    }

    pub fn with_identifier_parameters(mut self, has_identifier_parameters: bool) -> Self {
        self.has_identifier_parameters = has_identifier_parameters;
        self
    }
}

/// A top-level statement that doesn't parse, e.g. because it uses syntax of a newer Postgres.
#[derive(Debug, Clone)]
pub struct UnparsableStatement {
    pub range: TextRange,
    pub sql: String,
}
