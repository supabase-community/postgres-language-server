use core::slice;
use std::{fmt::Write, fs::read_to_string, path::Path};

use pgt_analyse::{AnalyserOptions, AnalysisFilter, RuleDiagnostic, RuleFilter};
use pgt_analyser::{AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams};
use pgt_console::StdDisplay;
use pgt_diagnostics::PrintDiagnostic;

pgt_test_macros::gen_tests! {
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
        read_to_string(full_path).unwrap_or_else(|_| panic!("Failed to read file: {} ", full_path));

    let ast = pgt_query::parse(&query).expect("failed to parse SQL");
    let options = AnalyserOptions::default();
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter,
    });

    let stmt = AnalysableStatement {
        root: ast.into_root(),
        range: pgt_text_size::TextRange::new(0.into(), u32::try_from(query.len()).unwrap().into()),
    };

    let results = analyser.run(AnalyserParams {
        stmts: vec![stmt],
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
    AnyDiagnostics,
    OnlyOne(String),
}

impl Expectation {
    fn from_file(content: &str) -> Self {
        for line in content.lines() {
            if line.contains("expect_no_diagnostics") {
                return Self::NoDiagnostics;
            }

            if line.contains("expect_only_") {
                let kind = line
                    .splitn(3, "_")
                    .last()
                    .expect("Use pattern: `-- expect_only_<category>`")
                    .trim();

                return Self::OnlyOne(kind.into());
            }
        }

        Self::AnyDiagnostics
    }

    fn assert(&self, diagnostics: &[RuleDiagnostic]) {
        match self {
            Self::NoDiagnostics => {
                if !diagnostics.is_empty() {
                    panic!("This test should not have any diagnostics.");
                }
            }
            Self::OnlyOne(category) => {
                let found_kinds = diagnostics
                    .iter()
                    .map(|d| d.get_category_name())
                    .collect::<Vec<&str>>()
                    .join(", ");

                if diagnostics.len() != 1 || diagnostics[0].get_category_name() != category {
                    panic!(
                        "This test should only have one diagnostic of kind: {category}\nReceived: {found_kinds}"
                    );
                }
            }
            Self::AnyDiagnostics => {}
        }
    }
}
