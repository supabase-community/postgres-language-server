//! Runs the type-check rules, and the lint rules that only report what Postgres rejects, over
//! Postgres' own regression suite of every supported version (`pgls_postgres_regress`), against
//! the catalog of a fresh database on that version.
//!
//! A finding on a statement Postgres accepted is a false positive and fails the test. Findings on
//! statements Postgres rejected are snapshotted per version, so a change in what we detect shows
//! up in review.
//!
//! The body of a `LANGUAGE sql` function is analysed like the workspace does it: right after the
//! `CREATE FUNCTION`, with the function's parameters in scope. A finding in the body of a function
//! Postgres accepted is a false positive too.

use std::{collections::BTreeMap, sync::Arc};

use pgls_analyse::{AnalysisFilter, RuleFilter};
use pgls_analyser::{
    AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams, LinterOptions, TYPECHECK_GROUP,
    UnparsableStatement,
};
use pgls_catalog::{CatalogBase, Snapshot, resolve::FunctionContext};
use pgls_console::StdDisplay;
use pgls_diagnostics::{Diagnostic, PrintDiagnostic};
use pgls_postgres_regress::{File, Verdict, Version};
use pgls_query::NodeEnum;
use pgls_text_size::TextRange;
use rayon::prelude::*;

/// Findings on statements Postgres accepted only because an earlier statement of the file failed.
/// The linter assumes every statement succeeds, like a migration that stops at the first error.
const KNOWN_ARTEFACTS: &[(&str, &str, &str)] = &[(
    "transactions",
    "writetest",
    "`DROP TABLE writetest` fails in a read-only session, so the table still exists",
)];

/// Known false positives in the bodies of SQL functions Postgres accepted: the file, a snippet of
/// the function, the rule, and the root cause. Remove an entry once it is fixed; an entry that
/// matches nothing fails the test.
const KNOWN_FALSE_POSITIVES: &[(&str, &str, &str, &str)] = &[(
    "polymorphism",
    "select array_prepend($1, $2)",
    "functionArgumentMismatch",
    "parameters of polymorphic types keep the pseudo-type in the body, so `$1 anyelement` doesn't \
     match `anycompatible`",
)];

#[test]
fn typecheck_reports_nothing_postgres_accepts() {
    let mut false_positives = Vec::new();
    let mut fixed = vec![true; KNOWN_FALSE_POSITIVES.len()];
    for version in pgls_postgres_regress::versions() {
        let catalog = catalog(&version);
        let files = version.files();
        let checked = files
            .par_iter()
            .map(|file| check(&catalog, file))
            .collect::<Vec<_>>();
        let analysed = Analysed {
            statements: files.iter().map(|x| x.statements.len()).sum(),
            bodies: checked.iter().map(|x| x.bodies).sum(),
        };
        let mut rejected = Vec::new();
        let mut artefacts = Vec::new();
        let mut known_false_positives = Vec::new();
        for finding in checked.into_iter().flat_map(|x| x.findings) {
            match &finding.verdict {
                Verdict::Accepted => {
                    if let Some(reason) = known_artefact(&finding) {
                        artefacts.push(format!("{}\n  {reason}", finding.location()));
                    } else if let Some(index) = known_false_positive(&finding) {
                        fixed[index] = false;
                        let (_, _, _, reason) = KNOWN_FALSE_POSITIVES[index];
                        known_false_positives.push(format!(
                            "{} {}\n  {reason}",
                            finding.location(),
                            finding.rule
                        ));
                    } else {
                        false_positives.push(format!(
                            "{}/{}: {}\n  {}\n  {}",
                            version.major,
                            finding.location(),
                            finding.rule,
                            finding.message,
                            truncate(&finding.sql, 300)
                        ));
                    }
                }
                Verdict::Rejected(state) => rejected.push((state.clone(), finding)),
                Verdict::Skipped => {}
            }
        }
        insta::assert_snapshot!(
            format!("typecheck_{}", version.major),
            report(
                &version,
                &analysed,
                &rejected,
                &artefacts,
                &known_false_positives
            )
        );
    }
    assert!(
        false_positives.is_empty(),
        "{} findings on statements Postgres accepted:\n\n{}",
        false_positives.len(),
        false_positives.join("\n\n")
    );
    let fixed = KNOWN_FALSE_POSITIVES
        .iter()
        .zip(fixed)
        .filter(|(_, fixed)| *fixed)
        .map(|((file, needle, rule, _), _)| format!("{file} {rule} `{needle}`"))
        .collect::<Vec<_>>();
    assert!(
        fixed.is_empty(),
        "known false positives that no longer occur, remove them from KNOWN_FALSE_POSITIVES:\n{}",
        fixed.join("\n")
    );
}

struct Analysed {
    statements: usize,
    /// The SQL function bodies analysed.
    bodies: usize,
}

struct Checked {
    findings: Vec<Finding>,
    bodies: usize,
}

struct Finding {
    file: String,
    /// Where the statement starts.
    line: usize,
    col: usize,
    /// Whether the finding is in the body of a SQL function.
    in_body: bool,
    rule: String,
    message: String,
    verdict: Verdict,
    sql: String,
}

impl Finding {
    fn location(&self) -> String {
        let body = if self.in_body { " (body)" } else { "" };
        format!("{}.sql:{}:{}{body}", self.file, self.line, self.col)
    }
}

fn catalog(version: &Version) -> Arc<CatalogBase> {
    let snapshot: Snapshot = serde_json::from_str(&version.catalog_json())
        .unwrap_or_else(|e| panic!("parse the catalog of {}: {e}", version.major));
    Arc::new(CatalogBase::new(Arc::new(snapshot)))
}

/// Lints a file like the workspace does: statements our parser doesn't understand (newer syntax)
/// are passed on as unparsable.
fn check(catalog: &Arc<CatalogBase>, file: &File) -> Checked {
    let options = LinterOptions::default();
    let rules = [
        RuleFilter::Group(TYPECHECK_GROUP),
        RuleFilter::Rule("correctness", "banDuplicateColumnAssignments"),
    ];
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter: AnalysisFilter::from_enabled_rules(&rules),
    });
    let mut stmts = Vec::new();
    let mut unparsable = Vec::new();
    let mut bodies = Vec::new();
    for statement in &file.statements {
        let sql = file.sql(statement);
        match parse(sql) {
            Ok(Some((root, has_identifier_parameters))) => {
                let body = sql_function_body(&root, sql, statement.range);
                stmts.push(
                    AnalysableStatement::new(root, statement.range)
                        .with_sql(sql)
                        .with_identifier_parameters(has_identifier_parameters),
                );
                if let Some(body) = body {
                    bodies.push(body.range);
                    stmts.push(body);
                }
            }
            Ok(None) => {}
            Err(()) => unparsable.push(UnparsableStatement {
                range: statement.range,
                sql: sql.to_owned(),
            }),
        }
    }
    let diagnostics = analyser.run(AnalyserParams {
        stmts,
        unparsable,
        catalog_base: Some(catalog.clone()),
        search_path: vec!["public".into()],
        typecheck: true,
        ..Default::default()
    });
    let findings = diagnostics
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.location().span.expect("findings have a span");
            let statement = file
                .statements
                .iter()
                .find(|x| x.range.contains_inclusive(span.start()))
                .expect("findings are inside a statement");
            let (line, col) = file.line_col(statement);
            Finding {
                file: file.name.clone(),
                line,
                col,
                in_body: bodies
                    .iter()
                    .any(|body| body.contains_inclusive(span.start())),
                rule: diagnostic
                    .get_category_name()
                    .trim_start_matches("lint/")
                    .to_owned(),
                message: format!("{}", StdDisplay(PrintDiagnostic::simple(diagnostic)))
                    .replace('\n', " "),
                verdict: statement.verdict.clone(),
                sql: file.sql(statement).to_owned(),
            }
        })
        .collect();
    Checked {
        findings,
        bodies: bodies.len(),
    }
}

/// The root of a statement, and whether it has psql identifier parameters. `Err` if it doesn't
/// parse, `None` if it isn't a single statement.
fn parse(sql: &str) -> Result<Option<(NodeEnum, bool)>, ()> {
    // psql variables (`:name`, `:'name'`) become positional parameters, as in the workspace.
    let conversion = pgls_lexer::convert_to_positional_params_with_metadata(sql);
    let ast = pgls_query::parse(&conversion.sql).map_err(|_| ())?;
    Ok(ast
        .into_root()
        .map(|root| (root, conversion.has_identifier_parameters)))
}

/// The statement in the body of a `LANGUAGE sql` function, as the workspace extracts and analyses
/// it.
fn sql_function_body(root: &NodeEnum, sql: &str, range: TextRange) -> Option<AnalysableStatement> {
    let NodeEnum::CreateFunctionStmt(create_fn) = root else {
        return None;
    };
    let body = pgls_query_ext::utils::sql_function_body(root, sql)?;
    let (body_root, has_identifier_parameters) = parse(&body.body).ok()??;
    Some(
        AnalysableStatement::new(body_root, body.range + range.start())
            .with_sql(body.body)
            .with_function(FunctionContext::from_create_function(create_fn))
            .with_identifier_parameters(has_identifier_parameters),
    )
}

fn known_false_positive(finding: &Finding) -> Option<usize> {
    KNOWN_FALSE_POSITIVES
        .iter()
        .position(|(file, needle, rule, _)| {
            finding.in_body
                && finding.file == *file
                && finding.rule == *rule
                && finding.sql.contains(needle)
        })
}

fn known_artefact(finding: &Finding) -> Option<&'static str> {
    KNOWN_ARTEFACTS
        .iter()
        .find(|(file, needle, _)| finding.file == *file && finding.sql.contains(needle))
        .map(|(_, _, reason)| *reason)
}

/// A summary per rule and SQLSTATE, the known artefacts and false positives, then one line per
/// finding on a rejected statement.
fn report(
    version: &Version,
    analysed: &Analysed,
    rejected: &[(String, Finding)],
    artefacts: &[String],
    false_positives: &[String],
) -> String {
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (state, finding) in rejected {
        *counts.entry((&finding.rule, state)).or_default() += 1;
    }
    let mut report = format!(
        "{}: {} statements and {} SQL function bodies, {} findings on statements Postgres rejected\n\n",
        version.tag,
        analysed.statements,
        analysed.bodies,
        rejected.len()
    );
    for ((rule, state), count) in counts {
        report.push_str(&format!("{rule} {state}: {count}\n"));
    }
    if !artefacts.is_empty() {
        report.push_str("\nKnown artefacts\n");
        for artefact in artefacts {
            report.push_str(&format!("{artefact}\n"));
        }
    }
    if !false_positives.is_empty() {
        report.push_str("\nKnown false positives\n");
        for false_positive in false_positives {
            report.push_str(&format!("{false_positive}\n"));
        }
    }
    report.push_str("\nFindings\n");
    let mut findings = rejected.iter().collect::<Vec<_>>();
    findings.sort_by(|(_, a), (_, b)| {
        (&a.file, a.line, a.col, &a.rule).cmp(&(&b.file, b.line, b.col, &b.rule))
    });
    for (state, finding) in findings {
        report.push_str(&format!(
            "{} {state} {}\n",
            finding.location(),
            finding.rule
        ));
    }
    report
}

fn truncate(sql: &str, max: usize) -> String {
    if sql.chars().count() <= max {
        sql.into()
    } else {
        format!("{}…", sql.chars().take(max).collect::<String>())
    }
}
