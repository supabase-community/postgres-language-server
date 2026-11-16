//! What a statement sees of the file: the catalog and the session after the statements before it.

use std::cell::OnceCell;

use pgls_catalog::{
    Catalog, Session,
    resolve::{Resolution, ResolveParams, resolve},
};
use pgls_query::NodeEnum;
use pgls_text_size::TextSize;

use crate::AnalysableStatement;

/// What the resolution of each statement collects, besides its findings and output.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolveOptions {
    /// Whether to collect the [`Resolution::references`].
    pub collect_references: bool,
    /// Whether to collect the [`Resolution::lookups`].
    pub collect_lookups: bool,
}

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
    resolve_options: ResolveOptions,
    /// The resolution of the current statement, once something asked for it.
    resolution: OnceCell<Resolution>,
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
            resolve_options: ResolveOptions::default(),
            resolution: OnceCell::new(),
        }
    }

    /// Sets what the resolution of each statement collects.
    pub fn with_resolve_options(mut self, resolve_options: ResolveOptions) -> Self {
        self.resolve_options = resolve_options;
        self
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

    /// The name resolution of the current statement `stmt` against the catalog and the search
    /// path as it sees them. It is computed once, on the first call, and shared by everything
    /// that looks at the statement: lint rules, the editor features, and the catalog, which gets
    /// the columns of the relation a view or `CREATE TABLE AS` creates from its output.
    ///
    /// Cost: one resolution of the statement, see [`resolve`].
    pub fn resolution(&self, stmt: &AnalysableStatement) -> &Resolution {
        debug_assert!(
            self.stmts
                .get(self.pos)
                .is_some_and(|current| std::ptr::eq(*current, &*stmt.root)),
            "the resolution is that of the current statement"
        );
        self.resolve(&stmt.root, stmt.function.as_ref(), stmt.sql.as_deref())
    }

    fn resolve(
        &self,
        stmt: &NodeEnum,
        function: Option<&pgls_catalog::resolve::FunctionContext>,
        sql: Option<&str>,
    ) -> &Resolution {
        self.resolution.get_or_init(|| {
            resolve(ResolveParams {
                stmt,
                catalog: &self.catalog,
                search_path: self.session.search_path(),
                function,
                sql,
                collect_references: self.resolve_options.collect_references,
                collect_lookups: self.resolve_options.collect_lookups,
            })
        })
    }

    /// Moves to the next statement.
    ///
    /// A statement that `runs` updates the session, and the catalog too if `updates_catalog`.
    /// Statements in the bodies of SQL functions are only analysed, not executed. A statement on
    /// placeholder names, like psql's `:name`, still takes its locks and changes the transaction,
    /// but the catalog can't follow it.
    ///
    /// A view, `CREATE TABLE AS` or `SELECT INTO` creates a relation with the output columns of
    /// its query, from the statement's [`Self::resolution`]. If nothing resolved the statement,
    /// it is resolved here. Statements in function bodies don't run, so this resolution has no
    /// function context.
    pub fn next(&mut self, runs: bool, updates_catalog: bool) {
        if runs && let Some(stmt) = self.stmts.get(self.pos) {
            if updates_catalog {
                let output = creates_relation_from_query(stmt)
                    .then(|| self.resolve(stmt, None, None).output.as_ref()?.columns())
                    .flatten();
                self.catalog
                    .apply(stmt, self.session.search_path(), output.as_deref());
            }
            self.session.apply(stmt, &self.catalog);
        }
        self.resolution = OnceCell::new();
        self.pos += 1;
    }
}

/// Whether `stmt` creates a relation whose columns are the output columns of its query.
fn creates_relation_from_query(stmt: &NodeEnum) -> bool {
    match stmt {
        NodeEnum::ViewStmt(_) | NodeEnum::CreateTableAsStmt(_) => true,
        NodeEnum::SelectStmt(select) => select.into_clause.is_some(),
        _ => false,
    }
}
