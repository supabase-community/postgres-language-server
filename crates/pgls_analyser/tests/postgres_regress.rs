//! Runs the type-check rules over Postgres' own regression suite of every supported version
//! (`pgls_postgres_regress`), against the catalog of a fresh database on that version.
//!
//! A finding on a statement Postgres accepted is a false positive and fails the test. Findings on
//! statements Postgres rejected are snapshotted per version, so a change in what we detect shows
//! up in review.

use std::{collections::BTreeMap, sync::Arc};

use pgls_analyse::{AnalysisFilter, RuleFilter};
use pgls_analyser::{
    AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams, LinterOptions, TYPECHECK_GROUP,
    UnparsableStatement,
};
use pgls_catalog::{CatalogBase, Snapshot};
use pgls_console::StdDisplay;
use pgls_diagnostics::{Diagnostic, PrintDiagnostic};
use pgls_postgres_regress::{File, Verdict, Version};
use rayon::prelude::*;

/// Findings on statements Postgres accepted only because an earlier statement of the file failed.
/// The linter assumes every statement succeeds, like a migration that stops at the first error.
const KNOWN_ARTEFACTS: &[(&str, &str, &str)] = &[(
    "transactions",
    "writetest",
    "`DROP TABLE writetest` fails in a read-only session, so the table still exists",
)];

#[test]
fn typecheck_reports_nothing_postgres_accepts() {
    let mut false_positives = Vec::new();
    for version in pgls_postgres_regress::versions() {
        let catalog = catalog(&version);
        let files = version.files();
        let findings = files
            .par_iter()
            .flat_map_iter(|file| check(&catalog, file))
            .collect::<Vec<_>>();
        let analysed = files.iter().map(|x| x.statements.len()).sum::<usize>();
        let mut rejected = Vec::new();
        let mut artefacts = Vec::new();
        for finding in findings {
            match &finding.verdict {
                Verdict::Accepted => match known_artefact(&finding) {
                    Some(reason) => artefacts.push(format!("{}\n  {reason}", finding.location())),
                    None => false_positives.push(format!(
                        "{}/{}: {}\n  {}\n  {}",
                        version.major,
                        finding.location(),
                        finding.rule,
                        finding.message,
                        truncate(&finding.sql, 300)
                    )),
                },
                Verdict::Rejected(state) => rejected.push((state.clone(), finding)),
                Verdict::Skipped => {}
            }
        }
        insta::assert_snapshot!(
            format!("typecheck_{}", version.major),
            report(&version, analysed, &rejected, &artefacts)
        );
    }
    assert!(
        false_positives.is_empty(),
        "{} findings on statements Postgres accepted:\n\n{}",
        false_positives.len(),
        false_positives.join("\n\n")
    );
}

struct Finding {
    file: String,
    /// Where the statement starts.
    line: usize,
    col: usize,
    rule: String,
    message: String,
    verdict: Verdict,
    sql: String,
}

impl Finding {
    fn location(&self) -> String {
        format!("{}.sql:{}:{}", self.file, self.line, self.col)
    }
}

fn catalog(version: &Version) -> Arc<CatalogBase> {
    let snapshot: Snapshot = serde_json::from_str(&version.catalog_json())
        .unwrap_or_else(|e| panic!("parse the catalog of {}: {e}", version.major));
    Arc::new(CatalogBase::new(Arc::new(snapshot)))
}

/// Lints a file like the workspace does: statements our parser doesn't understand (newer syntax)
/// are passed on as unparsable.
fn check(catalog: &Arc<CatalogBase>, file: &File) -> Vec<Finding> {
    let options = LinterOptions::default();
    let rules = [RuleFilter::Group(TYPECHECK_GROUP)];
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter: AnalysisFilter::from_enabled_rules(&rules),
    });
    let mut stmts = Vec::new();
    let mut unparsable = Vec::new();
    for statement in &file.statements {
        let sql = file.sql(statement);
        // psql variables (`:name`, `:'name'`) become positional parameters, as in the workspace.
        let conversion = pgls_lexer::convert_to_positional_params_with_metadata(sql);
        match pgls_query::parse(&conversion.sql) {
            Ok(ast) => stmts.extend(ast.into_root().map(|root| {
                AnalysableStatement::new(root, statement.range)
                    .with_sql(sql)
                    .with_identifier_parameters(conversion.has_identifier_parameters)
            })),
            Err(_) => unparsable.push(UnparsableStatement {
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
    diagnostics
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
        .collect()
}

fn known_artefact(finding: &Finding) -> Option<&'static str> {
    KNOWN_ARTEFACTS
        .iter()
        .find(|(file, needle, _)| finding.file == *file && finding.sql.contains(needle))
        .map(|(_, _, reason)| *reason)
}

/// A summary per rule and SQLSTATE, then one line per finding on a rejected statement.
fn report(
    version: &Version,
    analysed: usize,
    rejected: &[(String, Finding)],
    artefacts: &[String],
) -> String {
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (state, finding) in rejected {
        *counts.entry((&finding.rule, state)).or_default() += 1;
    }
    let mut report = format!(
        "{}: {analysed} statements, {} findings on statements Postgres rejected\n\n",
        version.tag,
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
