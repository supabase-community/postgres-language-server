use std::sync::{Arc, LazyLock};

use pgls_analyse::{AnalysisFilter, AppliesTo, MetadataRegistry};
use pgls_catalog::{CatalogBase, resolve::FunctionContext};
use pgls_diagnostics::DiagnosticTags;
use pgls_query_ext::function_body::{EmbeddedStatement, OffsetMap};
use pgls_text_size::{TextRange, TextSize};
pub use registry::visit_registry;

mod fix;
mod lint;
mod linter_context;
mod linter_options;
mod linter_registry;
mod linter_rule;
mod migration;
pub mod options;
mod registry;
mod walk;

// Re-export linter-specific types
pub use linter_context::{AnalysedFileContext, FileKind, LinterRuleContext, StatementContext};
pub use linter_options::{LinterOptions, LinterRules, RuleOptions};
pub use linter_registry::{
    LinterRegistryRuleParams, LinterRuleRegistry, LinterRuleRegistryBuilder,
};
pub use linter_rule::{FixEdit, LinterDiagnostic, LinterRule, RuleFix};
pub use migration::{Direction, MigrationSection};
use walk::is_plain_query;
pub use walk::{STOP_CHECK_INTERVAL, Walk, WalkParams, WalkState, walk_statements};

// For convenience in macros and rule files - keep these shorter names
pub use LinterDiagnostic as RuleDiagnostic;
pub use LinterRule as Rule;
pub use LinterRuleContext as RuleContext;
pub static METADATA: LazyLock<MetadataRegistry> = LazyLock::new(|| {
    let mut metadata = MetadataRegistry::default();
    // Use a separate visitor for metadata that implements pgls_analyse::RegistryVisitor
    visit_metadata_registry(&mut metadata);
    metadata
});

// Separate function for visiting metadata registry (uses pgls_analyse::RegistryVisitor)
fn visit_metadata_registry<V: pgls_analyse::RegistryVisitor>(registry: &mut V) {
    registry.record_category::<crate::lint::Lint>();
}

/// Main entry point to the analyser.
pub struct Analyser<'a> {
    /// Holds all rule options
    options: &'a LinterOptions,

    /// Holds all rules
    registry: LinterRuleRegistry,
}

/// The group of the typecheck rules. They only run when typechecking is enabled and a database
/// snapshot is available.
pub const TYPECHECK_GROUP: &str = "typecheck";

/// Released rules that were merged into another rule, with the rule that replaces them. Configs,
/// suppression comments, and rule selectors that name them keep working.
pub const REMOVED_RULES: &[(&str, &str)] = &[
    ("preferBigintOverInt", "preferBigInt"),
    ("preferBigintOverSmallint", "preferBigInt"),
    (
        "concurrentRefreshMatviewLock",
        "requireConcurrentRefreshMatview",
    ),
];

/// The rule that replaces a removed rule, if `name` is one.
pub fn replacement_of_removed_rule(name: &str) -> Option<&'static str> {
    REMOVED_RULES
        .iter()
        .find(|(removed, _)| *removed == name)
        .map(|(_, replacement)| *replacement)
}

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

#[derive(Default)]
pub struct AnalyserParams {
    pub stmts: Vec<AnalysableStatement>,
    /// The top-level statements that don't parse, in file order. Once one of them may have
    /// changed the catalog, nothing is known to be missing.
    pub unparsable: Vec<UnparsableStatement>,
    /// The database snapshot the catalog starts from. Without it, typecheck rules are silent.
    pub catalog_base: Option<Arc<CatalogBase>>,
    /// Explicit search path at the start of the file.
    pub search_path: Vec<String>,
    pub file_kind: FileKind,
    /// The sections of a migration file, like dbmate's up and down migrations. Each section is
    /// analysed as a migration of its own. Without sections, the whole file is one migration.
    pub sections: Vec<MigrationSection>,
    /// Whether the rules of the typecheck group run.
    pub typecheck: bool,
}

/// A top-level statement that doesn't parse, e.g. because it uses syntax of a newer Postgres.
#[derive(Debug, Clone)]
pub struct UnparsableStatement {
    pub range: TextRange,
    pub sql: String,
}

/// What the analyser learned about a single statement.
#[derive(Debug, Clone)]
pub struct StatementAnalysis {
    pub range: TextRange,
    /// `true` when typechecking is enabled and everything the statement references comes
    /// unchanged from the database, so checking it against the database is still correct.
    pub database_only: bool,
    /// `true` when a typecheck rule reported a diagnostic for this statement.
    pub has_typecheck_findings: bool,
    /// The explicit search path of the session at this statement.
    pub search_path: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AnalysisResult {
    pub diagnostics: Vec<LinterDiagnostic>,
    pub statements: Vec<StatementAnalysis>,
    /// Set when [`Analyser::analyse_until`] stopped before the end, so the result covers only
    /// the statements before that.
    pub stopped: bool,
}

pub struct AnalyserConfig<'a> {
    pub options: &'a LinterOptions,
    pub filter: AnalysisFilter<'a>,
}

impl<'a> Analyser<'a> {
    pub fn new(conf: AnalyserConfig<'a>) -> Self {
        let mut builder = LinterRuleRegistry::builder(&conf.filter);
        visit_registry(&mut builder);
        let registry = builder.build();

        Self {
            registry,
            options: conf.options,
        }
    }

    pub fn run(&self, params: AnalyserParams) -> Vec<LinterDiagnostic> {
        self.analyse(params).diagnostics
    }

    /// Runs the rules statement by statement, while the catalog and the session follow the
    /// effects of the statements, see [`walk_statements`].
    pub fn analyse(&self, params: AnalyserParams) -> AnalysisResult {
        self.analyse_until(params, &|| false)
    }

    /// Like [`Analyser::analyse`], but checks `should_stop` every [`STOP_CHECK_INTERVAL`]
    /// statements and stops when it returns `true`, e.g. because the document changed or the
    /// request was cancelled.
    pub fn analyse_until(
        &self,
        params: AnalyserParams,
        should_stop: &dyn Fn() -> bool,
    ) -> AnalysisResult {
        let mut result = AnalysisResult::default();

        let mut typecheck = params.typecheck && params.catalog_base.is_some();
        let snapshot = params
            .catalog_base
            .as_ref()
            .map(|base| base.snapshot().as_ref());

        let rules: Vec<_> = self
            .registry
            .rules
            .iter()
            .filter(|rule| {
                let runs_on_file =
                    rule.applies_to != AppliesTo::Migration || params.file_kind != FileKind::Other;
                let runs_typecheck = rule.group != TYPECHECK_GROUP || typecheck;
                runs_on_file && runs_typecheck
            })
            .collect();

        let walk = WalkParams {
            stmts: &params.stmts,
            unparsable: &params.unparsable,
            catalog_base: params.catalog_base.clone(),
            search_path: &params.search_path,
            sections: &params.sections,
            end: TextSize::from(u32::MAX),
            best_effort: false,
            from: None,
            checkpoints: &[],
            should_stop,
        };
        let walk = walk_statements(walk, |stmt, file_context| {
            let root = &*stmt.root;
            // The catalog can't follow statements on placeholder names, like `drop table :name`,
            // so the rest of the file isn't typechecked.
            if stmt.has_identifier_parameters && !is_plain_query(root) {
                typecheck = false;
            }
            let typecheck_statement = typecheck && !stmt.has_identifier_parameters;

            let statement =
                StatementContext::new(root, stmt.sql.as_deref(), stmt.function.as_ref());
            let mut has_typecheck_findings = false;

            let rule_params = LinterRegistryRuleParams {
                root,
                options: self.options,
                analysed_file_context: file_context,
                statement: &statement,
                snapshot,
            };
            for rule in &rules {
                if rule.group == TYPECHECK_GROUP && !typecheck_statement {
                    continue;
                }
                let diagnostics = (rule.run)(&rule_params);
                has_typecheck_findings |= rule.group == TYPECHECK_GROUP && !diagnostics.is_empty();

                result
                    .diagnostics
                    .extend(diagnostics.into_iter().map(|mut diagnostic| {
                        diagnostic.span = Some(match diagnostic.span {
                            Some(span) => stmt.document_range(span),
                            None => stmt.range,
                        });
                        // Fixes are computed on the statement text, which only matches the source
                        // if it is the source as written. A fix must keep the statement valid.
                        let fix_is_valid = stmt.sql.as_ref().is_some_and(|sql| {
                            stmt.offsets.is_verbatim()
                                && TextSize::of(sql.as_str()) == stmt.range.len()
                                && diagnostic
                                    .fix
                                    .as_ref()
                                    .is_none_or(|fix| keeps_statement_valid(fix, sql))
                        });
                        if !fix_is_valid {
                            diagnostic.fix = None;
                        }
                        if let Some(fix) = &mut diagnostic.fix {
                            fix.shift(stmt.range.start());
                            diagnostic.tags |= DiagnosticTags::FIXABLE;
                        }
                        diagnostic
                    }));
            }

            let database_only = typecheck_statement
                && statement
                    .resolution(file_context)
                    .is_some_and(|resolution| resolution.database_only);

            result.statements.push(StatementAnalysis {
                range: stmt.range,
                database_only,
                has_typecheck_findings,
                search_path: file_context.session().search_path().to_vec(),
            });
        });
        result.stopped = walk.stopped;

        result
    }
}

/// Whether the statement `sql` still parses with the fix applied. The edits are relative to the
/// statement. A fix that also edits outside of the statement, e.g. to add a `COMMIT` after it,
/// can't be checked on the statement alone and is kept.
fn keeps_statement_valid(fix: &RuleFix, sql: &str) -> bool {
    let len = TextSize::of(sql);
    if fix.edits.iter().any(|edit| edit.range.end() > len) {
        return true;
    }
    pgls_query::parse(&fix.apply(sql)).is_ok()
}

#[cfg(test)]
pub(crate) fn spanned_texts(group: &str, rule: &str, sql: &str) -> Vec<String> {
    let rule_filter = pgls_analyse::RuleFilter::Rule(group, rule);
    let options = LinterOptions::default();
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter: AnalysisFilter {
            enabled_rules: Some(std::slice::from_ref(&rule_filter)),
            ..Default::default()
        },
    });
    let root = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let range = TextRange::up_to(TextSize::of(sql));
    analyser
        .run(AnalyserParams {
            stmts: vec![AnalysableStatement::new(root, range).with_sql(sql)],
            ..Default::default()
        })
        .iter()
        .map(|diagnostic| {
            let span = pgls_diagnostics::Diagnostic::location(diagnostic)
                .span
                .unwrap();
            sql[span].to_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use core::slice;

    use crate::LinterOptions;
    use pgls_analyse::{AnalysisFilter, RuleFilter};
    use pgls_console::{
        Markup,
        fmt::{Formatter, Termcolor},
        markup,
    };
    use pgls_diagnostics::PrintDiagnostic;
    use pgls_text_size::TextRange;
    use termcolor::NoColor;

    use crate::{AnalysableStatement, Analyser, RuleFix, keeps_statement_valid};

    #[test]
    fn fixes_must_keep_the_statement_valid() {
        let sql = "create index users_email_idx on users (email)";
        let at = |offset: u32| pgls_text_size::TextSize::from(offset);
        assert!(keeps_statement_valid(
            &RuleFix::new("").insert(at(12), " concurrently"),
            sql
        ));
        assert!(!keeps_statement_valid(
            &RuleFix::new("").insert(at(12), " concurrently ("),
            sql
        ));
        // At the end of the statement, the edit is checked with it.
        assert!(keeps_statement_valid(
            &RuleFix::new("").insert(at(45), " where email is not null"),
            sql
        ));
        assert!(!keeps_statement_valid(
            &RuleFix::new("").insert(at(45), " where"),
            sql
        ));
        // After the statement, the fix can't be checked on the statement alone.
        assert!(keeps_statement_valid(
            &RuleFix::new("").insert(at(47), "\ncommit;"),
            sql
        ));
    }

    #[ignore]
    #[test]
    fn debug_test() {
        fn markup_to_string(markup: Markup) -> String {
            let mut buffer = Vec::new();
            let mut write = Termcolor(NoColor::new(&mut buffer));
            let mut fmt = Formatter::new(&mut write);
            fmt.write_markup(markup).unwrap();

            String::from_utf8(buffer).unwrap()
        }

        const SQL: &str = r#"alter table test drop column id;"#;
        let rule_filter = RuleFilter::Rule("destructive", "banDropColumn");

        let filter = AnalysisFilter {
            enabled_rules: Some(slice::from_ref(&rule_filter)),
            ..Default::default()
        };

        let ast = pgls_query::parse(SQL).expect("failed to parse SQL");
        let range = TextRange::new(0.into(), u32::try_from(SQL.len()).unwrap().into());

        let options = LinterOptions::default();

        let analyser = Analyser::new(crate::AnalyserConfig {
            options: &options,
            filter,
        });

        let results = analyser.run(crate::AnalyserParams {
            stmts: vec![AnalysableStatement::new(ast.into_root().unwrap(), range)],
            ..Default::default()
        });

        println!("*******************");
        for result in &results {
            let text = markup_to_string(markup! {
                {PrintDiagnostic::simple(result)}
            });
            eprintln!("{text}");
        }
        println!("*******************");

        // assert_eq!(results, vec![]);
    }

    #[test]
    fn analysis_stops_when_asked() {
        const SQL: &str = "alter table test drop column id;";
        let rule_filter = RuleFilter::Rule("destructive", "banDropColumn");
        let filter = AnalysisFilter {
            enabled_rules: Some(slice::from_ref(&rule_filter)),
            ..Default::default()
        };
        let options = LinterOptions::default();
        let analyser = Analyser::new(crate::AnalyserConfig {
            options: &options,
            filter,
        });
        let params = || crate::AnalyserParams {
            stmts: (0..200u32)
                .map(|i| {
                    let start = i * 40;
                    AnalysableStatement::new(
                        pgls_query::parse(SQL).unwrap().into_root().unwrap(),
                        TextRange::at(start.into(), u32::try_from(SQL.len()).unwrap().into()),
                    )
                })
                .collect(),
            ..Default::default()
        };

        let all = analyser.analyse(params());
        assert!(!all.stopped);
        assert_eq!(all.diagnostics.len(), 200);

        // Asked to stop at the second check.
        let checks = std::cell::Cell::new(0);
        let stopped = analyser.analyse_until(params(), &|| {
            checks.set(checks.get() + 1);
            checks.get() == 2
        });
        assert!(stopped.stopped);
        assert_eq!(stopped.diagnostics.len(), 127);
    }
}
