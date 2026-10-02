use std::sync::{Arc, LazyLock};

use pgls_analyse::{AnalysisFilter, AppliesTo, MetadataRegistry};
use pgls_catalog::{Catalog, CatalogBase, Session, resolve::FunctionContext};
use pgls_text_size::TextRange;
pub use registry::visit_registry;

mod lint;
mod linter_context;
mod linter_options;
mod linter_registry;
mod linter_rule;
pub mod options;
mod registry;

// Re-export linter-specific types
pub use linter_context::{AnalysedFileContext, FileKind, LinterRuleContext, StatementContext};
pub use linter_options::{LinterOptions, LinterRules, RuleOptions};
pub use linter_registry::{
    LinterRegistryRuleParams, LinterRuleRegistry, LinterRuleRegistryBuilder,
};
pub use linter_rule::{LinterDiagnostic, LinterRule};

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

#[derive(Debug)]
pub struct AnalysableStatement {
    pub root: pgls_query::NodeEnum,
    pub range: TextRange,
    /// The statement text, used for precise spans of typecheck findings.
    pub sql: Option<String>,
    /// Set when the statement is the body of a SQL function. Body statements are analysed, but
    /// they don't change the catalog or the session.
    pub function: Option<FunctionContext>,
    /// Set when identifiers of the statement are client-side parameters, like psql's
    /// `:schema.table`. The parsed names are placeholders, so the statement isn't typechecked.
    pub has_identifier_parameters: bool,
}

impl AnalysableStatement {
    pub fn new(root: pgls_query::NodeEnum, range: TextRange) -> Self {
        Self {
            root,
            range,
            sql: None,
            function: None,
            has_identifier_parameters: false,
        }
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
    /// The database snapshot the catalog starts from. Without it, typecheck rules are silent.
    pub catalog_base: Option<Arc<CatalogBase>>,
    /// Explicit search path at the start of the file.
    pub search_path: Vec<String>,
    pub file_kind: FileKind,
    /// Whether the rules of the typecheck group run.
    pub typecheck: bool,
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
    /// effects of the statements.
    pub fn analyse(&self, params: AnalyserParams) -> AnalysisResult {
        let mut result = AnalysisResult::default();

        let mut typecheck = params.typecheck && params.catalog_base.is_some();
        let snapshot = params
            .catalog_base
            .as_ref()
            .map(|base| base.snapshot().as_ref());
        let roots: Vec<pgls_query::NodeEnum> =
            params.stmts.iter().map(|s| s.root.clone()).collect();
        let mut file_context = AnalysedFileContext::new(
            &roots,
            Catalog::new(params.catalog_base.clone()),
            Session::new(params.search_path),
        );

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

        for (i, stmt) in params.stmts.iter().enumerate() {
            // The catalog and session can't follow statements on placeholder names, like
            // `drop table :name`, so the rest of the file isn't typechecked.
            if stmt.has_identifier_parameters && !is_plain_query(&roots[i]) {
                typecheck = false;
            }
            let typecheck_statement = typecheck && !stmt.has_identifier_parameters;

            let statement =
                StatementContext::new(&roots[i], stmt.sql.as_deref(), stmt.function.as_ref());
            let mut has_typecheck_findings = false;

            let rule_params = LinterRegistryRuleParams {
                root: &roots[i],
                options: self.options,
                analysed_file_context: &file_context,
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
                            Some(span) => span + stmt.range.start(),
                            None => stmt.range,
                        });
                        diagnostic
                    }));
            }

            let database_only = typecheck_statement
                && statement
                    .resolution(&file_context)
                    .is_some_and(|resolution| resolution.database_only);

            result.statements.push(StatementAnalysis {
                range: stmt.range,
                database_only,
                has_typecheck_findings,
                search_path: file_context.session().search_path().to_vec(),
            });

            file_context.next(stmt.function.is_none());
        }

        result
    }
}

/// Whether the statement only reads or writes rows, without changing the catalog or the
/// session.
fn is_plain_query(root: &pgls_query::NodeEnum) -> bool {
    use pgls_query::NodeEnum;
    match root {
        NodeEnum::SelectStmt(select) => select.into_clause.is_none(),
        NodeEnum::InsertStmt(_)
        | NodeEnum::UpdateStmt(_)
        | NodeEnum::DeleteStmt(_)
        | NodeEnum::MergeStmt(_) => true,
        _ => false,
    }
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

    use crate::{AnalysableStatement, Analyser};

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
