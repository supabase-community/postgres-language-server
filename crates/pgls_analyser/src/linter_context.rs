use std::cell::OnceCell;

use pgls_analyse::{GroupCategory, RuleCategory, RuleGroup, RuleMetadata};
use pgls_catalog::Snapshot;
use pgls_catalog::{
    Catalog, Session,
    resolve::{Finding, FunctionContext, Resolution, ResolveParams, resolve},
};

use pgls_query::protobuf::{ScanToken, Token};

use crate::fix::StatementTokens;
use crate::linter_rule::LinterRule;
use pgls_text_size::TextSize;

/// The kind of file being analysed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FileKind {
    /// No migrations directory is configured, so every rule runs.
    #[default]
    Unknown,
    /// The file is inside the configured migrations directory.
    Migration,
    /// A migrations directory is configured, but the file is not inside it.
    Other,
}

pub struct LinterRuleContext<'a, R: LinterRule> {
    stmt: &'a pgls_query::NodeEnum,
    options: &'a R::Options,
    snapshot: Option<&'a Snapshot>,
    file_context: &'a AnalysedFileContext<'a>,
    statement: &'a StatementContext<'a>,
}

impl<'a, R> LinterRuleContext<'a, R>
where
    R: LinterRule + Sized + 'static,
{
    pub fn new(
        stmt: &'a pgls_query::NodeEnum,
        options: &'a R::Options,
        snapshot: Option<&'a Snapshot>,
        file_context: &'a AnalysedFileContext,
        statement: &'a StatementContext<'a>,
    ) -> Self {
        Self {
            stmt,
            options,
            snapshot,
            file_context,
            statement,
        }
    }

    /// Returns the group that belongs to the current rule
    pub fn group(&self) -> &'static str {
        <R::Group as RuleGroup>::NAME
    }

    /// Returns the category that belongs to the current rule
    pub fn category(&self) -> RuleCategory {
        <<R::Group as RuleGroup>::Category as GroupCategory>::CATEGORY
    }

    /// Returns the AST root
    pub fn stmt(&self) -> &pgls_query::NodeEnum {
        self.stmt
    }

    pub fn file_context(&self) -> &AnalysedFileContext<'_> {
        self.file_context
    }

    /// The text of the current statement, if the caller provided it.
    pub fn sql(&self) -> Option<&str> {
        self.statement.sql
    }

    pub fn snapshot(&self) -> Option<&Snapshot> {
        self.snapshot
    }

    /// The catalog as it is before the current statement: the database snapshot plus the
    /// changes made by the statements before it.
    pub fn catalog(&self) -> &Catalog {
        self.file_context.catalog()
    }

    /// The session state (search path, transaction, locks, timeouts) before the current
    /// statement.
    pub fn session(&self) -> &Session {
        self.file_context.session()
    }

    /// Name resolution of the current statement against the catalog.
    ///
    /// Returns `None` without a database snapshot: typecheck rules only run with a database
    /// connection. The resolution is computed once per statement and shared by all rules.
    pub fn resolution(&self) -> Option<&Resolution> {
        self.statement.resolution(self.file_context)
    }

    /// The findings of the name resolution of the current statement, see [`Self::resolution`].
    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        self.resolution()
            .into_iter()
            .flat_map(|resolution| &resolution.findings)
    }

    /// The SQL function whose body the current statement is in, if it is in one.
    pub fn function(&self) -> Option<&FunctionContext> {
        self.statement.function
    }

    /// Whether the current statement runs in a transaction block. A statement in a function body
    /// runs in the transaction of the statement that calls the function. Otherwise, see
    /// [`AnalysedFileContext::in_transaction`].
    pub fn in_transaction(&self) -> bool {
        self.statement.function.is_some() || self.file_context.in_transaction()
    }

    /// Whether [`Self::in_transaction`] is known, rather than assumed because we don't know how
    /// the migration tool runs the file.
    pub fn knows_transaction(&self) -> bool {
        self.statement.function.is_some() || self.session().in_transaction().is_some()
    }

    /// The tokens of the current statement, to compute the edits of fixes. `None` if the
    /// statement text isn't known.
    pub fn tokens(&self) -> Option<StatementTokens<'_>> {
        self.statement.tokens()
    }

    /// Returns the metadata of the rule
    ///
    /// The metadata contains information about the rule, such as the name, version, language, and whether it is recommended.
    pub fn metadata(&self) -> &RuleMetadata {
        &R::METADATA
    }

    /// It retrieves the options that belong to a rule, if they exist.
    ///
    /// In order to retrieve a typed data structure, you have to create a deserializable
    /// data structure and define it inside the generic type `type Options` of the [LinterRule]
    ///
    pub fn options(&self) -> &R::Options {
        self.options
    }
}

/// Per-statement state shared by all rules that run on a statement.
pub struct StatementContext<'a> {
    stmt: &'a pgls_query::NodeEnum,
    sql: Option<&'a str>,
    function: Option<&'a FunctionContext>,
    resolution: OnceCell<Option<Resolution>>,
    /// The tokens of the statement without its comments, and the comments.
    tokens: OnceCell<Option<(Vec<ScanToken>, Vec<ScanToken>)>>,
}

impl<'a> StatementContext<'a> {
    pub fn new(
        stmt: &'a pgls_query::NodeEnum,
        sql: Option<&'a str>,
        function: Option<&'a FunctionContext>,
    ) -> Self {
        Self {
            stmt,
            sql,
            function,
            resolution: OnceCell::new(),
            tokens: OnceCell::new(),
        }
    }

    pub(crate) fn tokens(&self) -> Option<StatementTokens<'_>> {
        let sql = self.sql?;
        let (tokens, comments) = self
            .tokens
            .get_or_init(|| {
                let scan = pgls_query::scan(sql).ok()?;
                Some(scan.tokens.into_iter().partition(|token| {
                    !matches!(token.token(), Token::SqlComment | Token::CComment)
                }))
            })
            .as_ref()?;
        Some(StatementTokens::new(sql, tokens, comments))
    }

    pub fn resolution(&self, file_context: &AnalysedFileContext) -> Option<&Resolution> {
        self.resolution
            .get_or_init(|| {
                if !file_context.catalog().has_base() {
                    return None;
                }

                Some(resolve(ResolveParams {
                    stmt: self.stmt,
                    catalog: file_context.catalog(),
                    search_path: file_context.session().search_path(),
                    function: self.function,
                    sql: self.sql,
                    collect_references: false,
                }))
            })
            .as_ref()
    }
}

/// State of the file being analysed, as seen by the current statement.
///
/// In a file with migration sections, the file context is that of the current section.
pub struct AnalysedFileContext<'a> {
    pub stmts: &'a [pgls_query::NodeEnum],
    pos: usize,
    stmt_count: usize,
    catalog: Catalog,
    session: Session,
    /// Where each statement starts, and where the last statement ends, including the ones that
    /// don't parse.
    starts: Vec<TextSize>,
    end: Option<TextSize>,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a [pgls_query::NodeEnum], catalog: Catalog, session: Session) -> Self {
        Self {
            stmts,
            pos: 0,
            stmt_count: stmts.len(),
            catalog,
            session,
            starts: Vec::new(),
            end: None,
        }
    }

    /// Sets where the statements start in the file, and where the last one ends.
    pub fn with_positions(mut self, starts: Vec<TextSize>, end: TextSize) -> Self {
        self.starts = starts;
        self.end = Some(end);
        self
    }

    /// Where the last statement of the file, or of the section, ends, relative to the start of
    /// the current statement. Fixes can insert text there.
    pub fn end_offset(&self) -> Option<TextSize> {
        self.end?.checked_sub(*self.starts.get(self.pos)?)
    }

    /// Sets the number of statements of the file, if it differs from the parsed statements, e.g.
    /// because some don't parse.
    pub fn with_stmt_count(mut self, stmt_count: usize) -> Self {
        self.stmt_count = stmt_count;
        self
    }

    pub fn previous_stmts(&self) -> &[pgls_query::NodeEnum] {
        &self.stmts[0..self.pos]
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
    /// Executed statements (`applies_effects`) update the catalog and the session. Statements
    /// in the bodies of SQL functions are only analysed, not executed.
    pub fn next(&mut self, applies_effects: bool) {
        if applies_effects && self.pos < self.stmts.len() {
            let stmt = &self.stmts[self.pos];
            self.catalog.apply(stmt, self.session.search_path());
            self.session.apply(stmt);
        }
        self.pos += 1;
    }
}
