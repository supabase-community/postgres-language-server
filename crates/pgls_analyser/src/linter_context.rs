use std::cell::OnceCell;

use pgls_analyse::{GroupCategory, RuleCategory, RuleGroup, RuleMetadata};
use pgls_catalog::{
    Catalog, Session,
    resolve::{Finding, FunctionContext, Resolution, ResolveParams, resolve},
};

use pgls_query::protobuf::{RangeVar, ScanToken, Token};

use crate::fix::StatementTokens;
use crate::linter_rule::LinterRule;
use pgls_walk::AnalysedFileContext;

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
        file_context: &'a AnalysedFileContext,
        statement: &'a StatementContext<'a>,
    ) -> Self {
        Self {
            stmt,
            options,
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

    /// Whether the relation was created earlier in the current transaction. Other transactions
    /// can't see it until the transaction commits, so its locks, scans and rewrites block no
    /// one. A relation that an earlier transaction created and committed isn't new. The name is
    /// resolved with the catalog, see [`Session::has_created_relation`].
    pub fn is_new_relation(&self, relation: &RangeVar) -> bool {
        self.session()
            .has_created_relation(relation, self.catalog())
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
