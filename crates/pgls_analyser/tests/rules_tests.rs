use core::slice;
use std::{collections::HashMap, fmt::Write, fs::read_to_string, path::Path};

use pgls_analyse::{AnalyserOptions, AnalysisFilter, RuleDiagnostic, RuleFilter};
use pgls_analyser::{AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams};
use pgls_console::StdDisplay;
use pgls_diagnostics::PrintDiagnostic;

pgls_test_macros::gen_tests! {
  "tests/specs/**/*.sql",
  crate::rule_test
}

fn rule_test(full_path: &'static str, _: &str, _: &str) {
    let input_file = Path::new(full_path);

    let (group, rule, fname) = parse_test_path(input_file);

    let rule_filter = RuleFilter::Rule(group.as_str(), rule.as_str());
    let filter = AnalysisFilter {
        enabled_rules: Some(slice::from_ref(&rule_filter)),
        ..Default::default()
    };

    let query =
        read_to_string(full_path).unwrap_or_else(|_| panic!("Failed to read file: {full_path} "));

    let options = AnalyserOptions::default();
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter,
    });

    let split = pgls_statement_splitter::split(&query);

    let stmts = split
        .ranges
        .iter()
        .map(|r| {
            let text = &query[*r];
            let ast = pgls_query::parse(text).expect("failed to parse SQL");

            AnalysableStatement {
                root: ast.into_root().expect("Failed to convert AST to root node"),
                range: *r,
            }
        })
        .collect::<Vec<_>>();

    let results = analyser.run(AnalyserParams {
        stmts,
        schema_cache: None,
    });

    let mut snapshot = String::new();
    write_snapshot(&mut snapshot, query.as_str(), results.as_slice());

    insta::with_settings!({
        prepend_module_to_snapshot => false,
        snapshot_path => input_file.parent().unwrap(),
    }, {
        insta::assert_snapshot!(fname, snapshot);
    });

    let expectation = Expectation::from_file(&query);
    expectation.assert(results.as_slice());
}

fn parse_test_path(path: &Path) -> (String, String, String) {
    let mut comps: Vec<&str> = path
        .components()
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect();

    let fname = comps.pop().unwrap();
    let rule = comps.pop().unwrap();
    let group = comps.pop().unwrap();

    (group.into(), rule.into(), fname.into())
}

fn write_snapshot(snapshot: &mut String, query: &str, diagnostics: &[RuleDiagnostic]) {
    writeln!(snapshot, "# Input").unwrap();
    writeln!(snapshot, "```").unwrap();
    writeln!(snapshot, "{query}").unwrap();
    writeln!(snapshot, "```").unwrap();
    writeln!(snapshot).unwrap();

    if !diagnostics.is_empty() {
        writeln!(snapshot, "# Diagnostics").unwrap();
        for diagnostic in diagnostics {
            let printer = PrintDiagnostic::simple(diagnostic);

            writeln!(snapshot, "{}", StdDisplay(printer)).unwrap();
            writeln!(snapshot).unwrap();
        }
    }
}

enum Expectation {
    NoDiagnostics,
    Diagnostics(Vec<(String, usize)>),
}

impl Expectation {
    fn from_file(content: &str) -> Self {
        let mut multiple_of: HashMap<&str, i32> = HashMap::new();
        for line in content.lines() {
            if line.contains("expect_no_diagnostics") {
                if !multiple_of.is_empty() {
                    panic!(
                        "Cannot use both `expect_no_diagnostics` and `expect_` in the same test"
                    );
                }
                return Self::NoDiagnostics;
            }

            if line.contains("expect_") && !line.contains("expect_no_diagnostics") {
                let kind = line
                    .splitn(3, "_")
                    .last()
                    .expect("Use pattern: `-- expect_<category>`")
                    .trim();

                *multiple_of.entry(kind).or_insert(0) += 1;
            }
        }

        if !multiple_of.is_empty() {
            return Self::Diagnostics(
                multiple_of
                    .into_iter()
                    .map(|(k, v)| (k.into(), v as usize))
                    .collect(),
            );
        }

        panic!(
            "No expectation found in the test file. Use `-- expect_no_diagnostics` or `-- expect_<category>`"
        );
    }

    fn assert(&self, diagnostics: &[RuleDiagnostic]) {
        match self {
            Self::NoDiagnostics => {
                if !diagnostics.is_empty() {
                    panic!("This test should not have any diagnostics.");
                }
            }
            Self::Diagnostics(expected) => {
                let mut counts: HashMap<&str, usize> = HashMap::new();
                for diag in diagnostics {
                    *counts.entry(diag.get_category_name()).or_insert(0) += 1;
                }

                for (kind, expected_count) in expected {
                    let actual_count = counts.get(kind.as_str()).copied().unwrap_or(0);
                    if actual_count != *expected_count {
                        panic!(
                            "Expected {expected_count} diagnostics of kind `{kind}`, but found {actual_count}."
                        );
                    }
                }
            }
        }
    }
}
