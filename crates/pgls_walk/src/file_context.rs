//! What a statement sees of the file: the catalog and the session after the statements before it.

use pgls_catalog::{Catalog, Session};
use pgls_text_size::TextSize;

/// State of the file being analysed, as seen by the current statement.
///
/// In a file with migration sections, the file context is that of the current section.
pub struct AnalysedFileContext<'a> {
    pub stmts: &'a [&'a pgls_query::NodeEnum],
    pos: usize,
    stmt_count: usize,
    catalog: Catalog,
    session: Session,
    /// Where each statement starts, and where the last statement ends, including the ones that
    /// don't parse.
    starts: Vec<TextSize>,
    end: Option<TextSize>,
    /// Whether the last statement ends with a semicolon.
    terminated: bool,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a [&'a pgls_query::NodeEnum], catalog: Catalog, session: Session) -> Self {
        Self {
            stmts,
            pos: 0,
            stmt_count: stmts.len(),
            catalog,
            session,
            starts: Vec::new(),
            end: None,
            terminated: true,
        }
    }

    /// Sets where the statements start in the file, where the last one ends, and whether it ends
    /// with a semicolon.
    pub fn with_positions(
        mut self,
        starts: Vec<TextSize>,
        end: TextSize,
        terminated: bool,
    ) -> Self {
        self.starts = starts;
        self.end = Some(end);
        self.terminated = terminated;
        self
    }

    /// Where to insert what, relative to the start of the current statement, to add the
    /// statement `sql` after the last statement of the file, or of the section: on a new line,
    /// after a semicolon if the last statement doesn't end with one.
    pub fn append_statement(&self, sql: &str) -> Option<(TextSize, String)> {
        let offset = self.end?.checked_sub(*self.starts.get(self.pos)?)?;
        let separator = if self.terminated { "\n" } else { ";\n" };
        Some((offset, format!("{separator}{sql}")))
    }

    /// Sets the number of statements of the file, if it differs from the parsed statements, e.g.
    /// because some don't parse.
    pub fn with_stmt_count(mut self, stmt_count: usize) -> Self {
        self.stmt_count = stmt_count;
        self
    }

    /// The statements before the current one.
    pub fn previous_stmts(&self) -> &[&'a pgls_query::NodeEnum] {
        self.stmts.get(..self.pos).unwrap_or(self.stmts)
    }

    /// The statements after the current one.
    pub fn later_stmts(&self) -> &[&'a pgls_query::NodeEnum] {
        self.stmts.get(self.pos + 1..).unwrap_or_default()
    }

    /// The number of statements that the file runs, including those that don't parse.
    /// Statements in function bodies don't count.
    pub fn stmt_count(&self) -> usize {
        self.stmt_count
    }

    /// Whether the next statement runs in a transaction block. Without migration sections we
    /// don't know whether the migration tool runs the file in a transaction. Migration tools
    /// usually do, so a file with more than one statement is assumed to.
    pub fn in_transaction(&self) -> bool {
        self.session
            .in_transaction()
            .unwrap_or_else(|| self.stmt_count() > 1)
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The catalog after the statements.
    pub fn into_catalog(self) -> Catalog {
        self.catalog
    }

    /// Records a statement that may have changed the catalog in ways it can't model, e.g. one
    /// that doesn't parse.
    pub fn taint_catalog(&mut self) {
        self.catalog.taint();
    }

    /// The next statements are part of a query with more than one statement, see
    /// [`Session::begin_implicit_block`].
    pub fn begin_implicit_block(&mut self) {
        self.session.begin_implicit_block();
    }

    /// The query started with [`Self::begin_implicit_block`] ended.
    pub fn end_implicit_block(&mut self) {
        self.session.end_implicit_block();
    }

    /// Moves to the next statement.
    ///
    /// A statement that `runs` updates the session, and the catalog too if `updates_catalog`.
    /// Statements in the bodies of SQL functions are only analysed, not executed. A statement on
    /// placeholder names, like psql's `:name`, still takes its locks and changes the transaction,
    /// but the catalog can't follow it.
    pub fn next(&mut self, runs: bool, updates_catalog: bool) {
        if runs && let Some(stmt) = self.stmts.get(self.pos) {
            if updates_catalog {
                self.catalog.apply(stmt, self.session.search_path());
            }
            self.session.apply(stmt, &self.catalog);
        }
        self.pos += 1;
    }
}
