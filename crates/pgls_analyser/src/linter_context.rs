use std::cell::OnceCell;

use pgls_analyse::{GroupCategory, RuleCategory, RuleGroup, RuleMetadata};
use pgls_catalog::Snapshot;
use pgls_catalog::{
    Catalog, Session,
    resolve::{FunctionContext, Resolution, ResolveParams, resolve},
};

pub(crate) use pgls_catalog::{is_reindex_concurrent, is_vacuum_full};

use crate::linter_rule::LinterRule;

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
        }
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
                }))
            })
            .as_ref()
    }
}

/// State of the file being analysed, as seen by the current statement.
pub struct AnalysedFileContext<'a> {
    pub stmts: &'a Vec<pgls_query::NodeEnum>,
    pos: usize,
    catalog: Catalog,
    session: Session,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a Vec<pgls_query::NodeEnum>, catalog: Catalog, session: Session) -> Self {
        Self {
            stmts,
            pos: 0,
            catalog,
            session,
        }
    }

    pub fn previous_stmts(&self) -> &[pgls_query::NodeEnum] {
        &self.stmts[0..self.pos]
    }

    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Records a statement that may have changed the catalog in ways it can't model, e.g. one
    /// that doesn't parse.
    pub fn taint_catalog(&mut self) {
        self.catalog.taint();
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
