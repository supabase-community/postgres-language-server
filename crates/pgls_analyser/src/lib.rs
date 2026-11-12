use std::{
    ops::Range,
    sync::{Arc, LazyLock},
};

use pgls_analyse::{AnalysisFilter, AppliesTo, MetadataRegistry};
use pgls_catalog::CatalogBase;
use pgls_diagnostics::DiagnosticTags;
use pgls_text_size::{TextRange, TextSize};
pub use registry::visit_registry;

mod fix;
mod lint;
mod linter_context;
mod linter_options;
mod linter_registry;
mod linter_rule;
pub mod options;
mod registry;
mod transaction_block;
mod utils;

// Re-export linter-specific types
pub use linter_context::{FileKind, LinterRuleContext, StatementContext};
pub use linter_options::{LinterOptions, LinterRules, RuleOptions};
pub use linter_registry::{
    LinterRegistryRuleParams, LinterRuleRegistry, LinterRuleRegistryBuilder,
};
pub use linter_rule::{FixEdit, LinterDiagnostic, LinterRule, RuleFix};
pub use pgls_walk::{
    AnalysableStatement, AnalysedFileContext, Direction, MigrationSection, UnparsableStatement,
    WalkParams, is_plain_query, walk_statements,
};

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
    /// The text of the file. Fixes that also edit after their statement, like the `COMMIT` that
    /// `banUncommittedTransaction` adds after the last statement, are checked on it. Without
    /// it, such fixes are kept unchecked.
    pub source: Option<String>,
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
    /// `true` when `banDuplicateColumnAssignments` reported the statement. Postgres reports
    /// the same assignments when it prepares the statement, without a position.
    pub has_duplicate_assignments: bool,
    /// The explicit search path of the session at this statement.
    pub search_path: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AnalysisResult {
    pub diagnostics: Vec<LinterDiagnostic>,
    pub statements: Vec<StatementAnalysis>,
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
        let mut result = AnalysisResult::default();

        let mut typecheck = params.typecheck && params.catalog_base.is_some();
        let file = params.source.as_deref().map(|source| FileText {
            source,
            stmts: params
                .stmts
                .iter()
                .filter(|stmt| stmt.function.is_none())
                .map(|stmt| stmt.range)
                .chain(params.unparsable.iter().map(|stmt| stmt.range))
                .collect(),
        });

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
        };
        walk_statements(walk, |stmt, file_context| {
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
            let mut has_duplicate_assignments = false;

            let rule_params = LinterRegistryRuleParams {
                root,
                options: self.options,
                analysed_file_context: file_context,
                statement: &statement,
            };
            for rule in &rules {
                if rule.group == TYPECHECK_GROUP && !typecheck_statement {
                    continue;
                }
                let diagnostics = (rule.run)(&rule_params);
                has_typecheck_findings |= rule.group == TYPECHECK_GROUP && !diagnostics.is_empty();
                has_duplicate_assignments |= diagnostics.iter().any(|diagnostic| {
                    diagnostic.category
                        == pgls_diagnostics::category!("lint/banDuplicateColumnAssignments")
                });

                result
                    .diagnostics
                    .extend(diagnostics.into_iter().map(|mut diagnostic| {
                        debug_assert_in_statement(&diagnostic, stmt);
                        diagnostic.move_into_file(stmt);
                        // Fixes are computed on the statement text, which only matches the source
                        // if it is the source as written. A fix must keep the statement valid.
                        let fix_is_valid = stmt.sql.as_ref().is_some_and(|sql| {
                            stmt.offsets.is_verbatim()
                                && TextSize::of(sql.as_str()) == stmt.range.len()
                                && diagnostic.fix.as_ref().is_none_or(|fix| {
                                    keeps_sql_valid(fix, sql, stmt.range.start(), file.as_ref())
                                })
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
                has_duplicate_assignments,
                search_path: file_context.session().search_path().to_vec(),
            });
        });

        result
    }
}

/// The text of the file, and the ranges of its top-level statements.
struct FileText<'a> {
    source: &'a str,
    stmts: Vec<TextRange>,
}

/// Whether the SQL still parses with the fix applied. The edits are relative to the statement
/// `sql`, which starts at `start` in the file. A fix within the statement, or at its end, is
/// checked on the statement. A fix that also edits after the statement, e.g. to add a `COMMIT`
/// after the last statement, is checked on the text of the file from the start of the statement
/// to the end of the last statement it edits. If that text doesn't parse without the fix, or the
/// text of the file isn't known, the fix can't be checked and is kept.
///
/// Like the statements themselves, the SQL is parsed with its placeholders (psql variables such
/// as `:name`, `$name`) converted to parameters, see [pgls_lexer::convert_to_positional_params].
fn keeps_sql_valid(fix: &RuleFix, sql: &str, start: TextSize, file: Option<&FileText>) -> bool {
    let len = TextSize::of(sql);
    let Some(edits_end) = fix.edits.iter().map(|edit| edit.range.end()).max() else {
        return true;
    };
    if edits_end <= len {
        return parses(&fix.apply(sql));
    }
    let Some(file) = file else {
        return true;
    };
    let edits_end = start + edits_end;
    let end = file
        .stmts
        .iter()
        .filter(|stmt| stmt.start() >= start && stmt.start() < edits_end)
        .map(|stmt| stmt.end())
        .fold(edits_end, TextSize::max);
    let Some(text) = file.source.get(usize::from(start)..usize::from(end)) else {
        return true;
    };
    if !parses(text) {
        return true;
    }
    parses(&fix.apply(text))
}

/// Whether the SQL parses with its placeholders converted to parameters. The conversion keeps the
/// length of the text, so the fix edits apply to the source text unchanged.
fn parses(sql: &str) -> bool {
    pgls_query::parse(&pgls_lexer::convert_to_positional_params(sql)).is_ok()
}

/// The span, the detail spans and the fix edits of a rule's diagnostic are relative to the
/// statement and must lie on char boundaries of its text. Fix edits may also start at or after
/// its end, e.g. to add a `COMMIT` after it. They are moved into the file next, and checked again
/// where they are used.
fn debug_assert_in_statement(diagnostic: &LinterDiagnostic, stmt: &AnalysableStatement) {
    if !cfg!(debug_assertions) {
        return;
    }
    let in_statement = |range: TextRange| match &stmt.sql {
        Some(sql) => sql.get(Range::<usize>::from(range)).is_some(),
        None => range.end() <= stmt.range.len(),
    };
    let category = diagnostic.category.name();
    let details = diagnostic
        .rule_advice
        .details
        .iter()
        .filter_map(|detail| detail.range);
    for span in diagnostic.span.into_iter().chain(details) {
        debug_assert!(
            in_statement(span),
            "{category}: span {span:?} is not inside the statement at {:?}",
            stmt.range
        );
    }
    let len = stmt.sql.as_deref().map_or(stmt.range.len(), TextSize::of);
    for edit in diagnostic.fix.iter().flat_map(|fix| &fix.edits) {
        debug_assert!(
            edit.range.start() >= len || in_statement(edit.range),
            "{category}: fix edit {:?} is not on char boundaries of the statement at {:?}",
            edit.range,
            stmt.range
        );
    }
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
            source: Some(sql.to_owned()),
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

    use crate::{
        AnalysableStatement, Analyser, FileText, LinterDiagnostic, RuleFix, keeps_sql_valid,
    };
    use pgls_diagnostics::category;

    #[test]
    fn fixes_must_keep_the_statement_valid() {
        let sql = "create index users_email_idx on users (email)";
        let at = |offset: u32| pgls_text_size::TextSize::from(offset);
        let valid = |fix: RuleFix| keeps_sql_valid(&fix, sql, at(0), None);
        assert!(valid(RuleFix::new("").insert(at(12), " concurrently")));
        assert!(!valid(RuleFix::new("").insert(at(12), " concurrently (")));
        // At the end of the statement, the edit is checked with it.
        assert!(valid(
            RuleFix::new("").insert(at(45), " where email is not null")
        ));
        assert!(!valid(RuleFix::new("").insert(at(45), " where")));
        // After the statement, the fix can't be checked without the file.
        assert!(valid(RuleFix::new("").insert(at(47), "\ncommit;")));
    }

    #[test]
    fn fixes_after_the_statement_are_checked_on_the_file() {
        let source = "begin;\ncreate table t (id int);\ncreate table u (id int)";
        let at = |offset: u32| pgls_text_size::TextSize::from(offset);
        let file = FileText {
            source,
            stmts: vec![
                TextRange::new(at(0), at(5)),
                TextRange::new(at(7), at(30)),
                TextRange::new(at(32), at(55)),
            ],
        };
        // A fix of `begin`.
        let valid = |offset: u32, text: &str| {
            let fix = RuleFix::new("").insert(at(offset), text);
            keeps_sql_valid(&fix, &source[..5], at(0), Some(&file))
        };
        // After `create table t (id int);`.
        assert!(valid(31, "\ncommit;"));
        assert!(!valid(30, "\ncommit"));
        // After the last statement, which has no semicolon.
        assert!(valid(55, ";\ncommit;"));
        assert!(!valid(55, "\ncommit;"));
        // Inside `create table t`.
        assert!(!valid(14, "\ncommit;"));
        // Text that doesn't parse can't be checked.
        let file = FileText {
            source: "begin;\ncreate tabel t (id int)",
            stmts: vec![TextRange::new(at(0), at(5)), TextRange::new(at(7), at(30))],
        };
        let fix = RuleFix::new("").insert(at(30), "\ncommit;");
        assert!(keeps_sql_valid(&fix, "begin", at(0), Some(&file)));
    }

    #[test]
    fn diagnostics_move_into_the_file() {
        let at = |offset: u32| pgls_text_size::TextSize::from(offset);
        let statement = AnalysableStatement::new(
            pgls_query::parse("select 1").unwrap().into_root().unwrap(),
            TextRange::new(at(10), at(30)),
        );
        let mut diagnostic = LinterDiagnostic::new(
            category!("lint/banDropColumn"),
            Some(TextRange::new(at(2), at(4))),
            "",
        )
        .detail(Some(TextRange::new(at(5), at(7))), "here")
        .detail(None, "no span");
        diagnostic.move_into_file(&statement);
        assert_eq!(diagnostic.span, Some(TextRange::new(at(12), at(14))));
        let details: Vec<_> = diagnostic
            .rule_advice
            .details
            .iter()
            .map(|detail| detail.range)
            .collect();
        assert_eq!(details, [Some(TextRange::new(at(15), at(17))), None]);

        // Without a span, the diagnostic points at the statement.
        let mut diagnostic = LinterDiagnostic::new(category!("lint/banDropColumn"), None, "");
        diagnostic.move_into_file(&statement);
        assert_eq!(diagnostic.span, Some(statement.range));
    }

    /// Placeholders are converted to parameters before the fixed statement is parsed, like
    /// before the statement itself is parsed.
    #[test]
    fn fixes_of_statements_with_placeholders_are_checked_with_parameters() {
        let at = |offset: u32| pgls_text_size::TextSize::from(offset);
        for sql in [
            "create index users_tenant_idx on users (email) where tenant = :tenant",
            "create index users_tenant_idx on users (email) where tenant = :'tenant'",
            "create index users_tenant_idx on users (email) where tenant = $tenant",
            "create index users_tenant_idx on :\"table\" (email)",
        ] {
            let valid = |fix: RuleFix| keeps_sql_valid(&fix, sql, at(0), None);
            assert!(
                valid(RuleFix::new("").insert(at(12), " concurrently")),
                "{sql}"
            );
            assert!(
                !valid(RuleFix::new("").insert(at(12), " concurrently (")),
                "{sql}"
            );
        }
    }

    /// A fix that edits after its statement is checked on the text of the file, with the
    /// placeholders of all statements in it converted to parameters.
    #[test]
    fn fixes_after_statements_with_placeholders_are_checked_with_parameters() {
        let source = "insert into t values (:'v');\nupdate :\"schema\".t set id = $id;";
        let at = |offset: usize| pgls_text_size::TextSize::from(u32::try_from(offset).unwrap());
        let first = TextRange::new(at(0), at(source.find(';').unwrap()));
        let second = TextRange::new(at(source.find("update").unwrap()), at(source.len() - 1));
        let file = FileText {
            source,
            stmts: vec![first, second],
        };
        // A fix of the first statement.
        let valid = |offset: usize, text: &str| {
            let fix = RuleFix::new("").insert(at(offset), text);
            keeps_sql_valid(&fix, &source[first], at(0), Some(&file))
        };
        // After the last statement.
        assert!(valid(source.len(), "\ncommit;"));
        // Before the semicolon of the last statement. Without the placeholders converted, the
        // text wouldn't parse without the fix either, and the fix would be kept.
        assert!(!valid(source.len() - 1, "\ncommit"));
    }

    /// The workspace drops Postgres' error for the statements that `banDuplicateColumnAssignments`
    /// reported, which has no position.
    #[test]
    fn statements_with_duplicate_assignments_are_marked() {
        let rule_filter = RuleFilter::Rule("correctness", "banDuplicateColumnAssignments");
        let options = LinterOptions::default();
        let analyser = Analyser::new(crate::AnalyserConfig {
            options: &options,
            filter: AnalysisFilter {
                enabled_rules: Some(slice::from_ref(&rule_filter)),
                ..Default::default()
            },
        });
        let marked = |sql: &str| {
            let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
            let range = TextRange::new(0.into(), u32::try_from(sql.len()).unwrap().into());
            let result = analyser.analyse(crate::AnalyserParams {
                stmts: vec![AnalysableStatement::new(ast, range)],
                ..Default::default()
            });
            result.statements[0].has_duplicate_assignments
        };
        assert!(marked(
            "merge into users u using imports i on u.id = i.id \
             when matched then update set name = i.name, name = 'again'"
        ));
        assert!(marked("update users set name = 'a', name = 'b'"));
        assert!(!marked("update users set name = 'a', id = 1"));
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
}
