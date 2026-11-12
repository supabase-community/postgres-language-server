//! Formats the snapshot fixtures and the Postgres regression suite under 32 combinations of the
//! formatter options, and checks that every output round-trips (`format_statement` refuses one
//! whose AST differs), is idempotent and has no stray whitespace or mixed indentation.
//!
//! The snapshot tests render each fixture only with the options in its header, and the regression
//! corpus test only with the defaults, so a bug that needs two options together, leading commas in
//! fit layout for instance, slips through both.
//!
//! Each corpus takes about half a minute in release and a few minutes in debug, so the tests are
//! ignored in the regular run. CI runs them in release when the formatter changes:
//!
//! ```sh
//! cargo test -p pgls_pretty_print --release --test option_matrix -- --ignored
//! ```

mod common;

use std::{
    collections::HashSet,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
};

use pgls_postgres_regress::Verdict;
use pgls_pretty_print::{
    CastStyle, ClauseBodyStyle, CommaStyle, FormatConfig, FormatError, IndentStyle, KeywordCase,
    Layout, LogicalOperatorPlacement, format_statement,
};
use rayon::prelude::*;

/// At most this many failures are printed.
const MAX_REPORTED: usize = 30;

/// Every combination of the options that change the layout, 32 in all. The indent and line width
/// cycle through all nine of their pairs, and the remaining options ride along with them.
fn option_matrix() -> Vec<(String, FormatConfig)> {
    let mut configs = Vec::new();
    for layout in [Layout::Fit, Layout::Expanded] {
        for clause_body_style in [ClauseBodyStyle::Break, ClauseBodyStyle::Compact] {
            for logical_operator_placement in [
                LogicalOperatorPlacement::Trailing,
                LogicalOperatorPlacement::Leading,
            ] {
                for comma_style in [CommaStyle::Trailing, CommaStyle::Leading] {
                    for values_row_layout in [Layout::Fit, Layout::Expanded] {
                        let pair = configs.len() % 9;
                        let (indent_style, indent_size) = match pair % 3 {
                            0 => (IndentStyle::Spaces, 2),
                            1 => (IndentStyle::Spaces, 4),
                            _ => (IndentStyle::Tabs, 4),
                        };
                        let line_width = [40, 80, 120][pair / 3];
                        let upper = line_width == 120;
                        let config = FormatConfig {
                            line_width,
                            indent_size,
                            indent_style: indent_style.clone(),
                            keyword_case: if upper {
                                KeywordCase::Upper
                            } else {
                                KeywordCase::Lower
                            },
                            constant_case: if upper {
                                KeywordCase::Upper
                            } else {
                                KeywordCase::Lower
                            },
                            cast_style: if matches!(indent_style, IndentStyle::Tabs) {
                                CastStyle::Operator
                            } else {
                                CastStyle::Cast
                            },
                            isolate_semicolon: line_width == 40,
                            comma_style: comma_style.clone(),
                            logical_operator_placement: logical_operator_placement.clone(),
                            layout: layout.clone(),
                            values_row_layout: values_row_layout.clone(),
                            clause_body_style: clause_body_style.clone(),
                            ..Default::default()
                        };
                        let name = format!(
                            "layout={layout:?} clauseBodyStyle={clause_body_style:?} \
                             logicalOperatorPlacement={logical_operator_placement:?} \
                             commaStyle={comma_style:?} valuesRowLayout={values_row_layout:?} \
                             indentStyle={indent_style:?} indentSize={indent_size} \
                             lineWidth={line_width}"
                        );
                        configs.push((name, config));
                    }
                }
            }
        }
    }
    configs
}

/// Every statement of the snapshot fixtures, without their `-- pgls-format:` headers.
fn fixture_statements() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let mut statements = Vec::new();
    for kind in ["single", "multi"] {
        let mut paths = fs::read_dir(root.join(kind))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "sql"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let content = fs::read_to_string(&path).unwrap();
            let content = match content.strip_prefix("-- pgls-format:") {
                Some(rest) => rest.split_once('\n').map_or("", |(_, sql)| sql).to_string(),
                None => content,
            };
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            // A `single` fixture is one statement, which the splitter can cut in the wrong place.
            if kind == "single" {
                statements.push((format!("{kind}/{name}"), content.trim().to_string()));
                continue;
            }
            for range in pgls_statement_splitter::split(&content).ranges {
                let sql = content[usize::from(range.start())..usize::from(range.end())].trim();
                statements.push((format!("{kind}/{name}"), sql.to_string()));
            }
        }
    }
    statements
}

/// Every statement Postgres accepted in its own regression suite, for the versions the parser
/// knows.
fn regression_statements() -> Vec<(String, String)> {
    let parser_major = pgls_query::parse("select 1").unwrap().protobuf.version as u32 / 10000;
    let mut statements = Vec::new();
    for version in pgls_postgres_regress::versions()
        .into_iter()
        .filter(|version| version.major <= parser_major)
    {
        for file in version.files() {
            for statement in &file.statements {
                if statement.verdict != Verdict::Accepted {
                    continue;
                }
                let (line, column) = file.line_col(statement);
                statements.push((
                    format!("{}/{}.sql:{line}:{column}", version.major, file.name),
                    file.sql(statement).to_string(),
                ));
            }
        }
    }
    statements
}

fn format(sql: &str, config: &FormatConfig) -> Result<Option<String>, String> {
    let Some(ast) = pgls_query::parse(sql)
        .map_err(|e| format!("doesn't parse: {e}"))?
        .into_root()
    else {
        return Ok(None);
    };
    match catch_unwind(AssertUnwindSafe(|| format_statement(&ast, sql, config))) {
        Ok(Ok(result)) => Ok(Some(result.formatted)),
        // A comment that can't be placed leaves the statement as written, by design.
        Ok(Err(FormatError::UnplaceableComment { .. })) => Ok(None),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("the formatter panicked".to_string()),
    }
}

fn check(sql: &str, config: &FormatConfig) -> Result<(), String> {
    let Some(output) = format(sql, config)? else {
        return Ok(());
    };
    let mut problems = common::whitespace_problems(&output);
    problems.extend(common::empty_statements(&output));
    // At width 40 some statements can't fit. A `::` cast doesn't break either, so in a compact
    // `select '...'::type` or `('...'::jsonb)['a']['b']` nothing can move to the next line.
    if config.line_width >= 80 && matches!(config.cast_style, CastStyle::Cast) {
        problems.extend(common::width_problems(
            &output,
            config.line_width,
            config.indent_size,
        ));
    }
    problems.extend(common::mixed_indentation(
        &output,
        matches!(config.indent_style, IndentStyle::Tabs),
    ));
    if !problems.is_empty() {
        return Err(format!("{}\n{output}", problems.join("\n")));
    }
    match format(&output, config) {
        Ok(Some(again)) if again == output => Ok(()),
        Ok(Some(again)) => Err(format!(
            "not idempotent\n{output}\n--- formatted again\n{again}"
        )),
        Ok(None) => Ok(()),
        Err(e) => Err(format!("formatting the output failed: {e}\n{output}")),
    }
}

fn assert_every_combination_survives(statements: &[(String, String)]) {
    // Statements repeat across fixtures and versions.
    let mut seen = HashSet::new();
    let statements = statements
        .iter()
        .filter(|(_, sql)| !sql.is_empty() && seen.insert(sql.as_str()))
        .collect::<Vec<_>>();
    assert!(!statements.is_empty(), "no statements to check");

    let configs = option_matrix();
    let failures = configs
        .par_iter()
        .flat_map_iter(|(name, config)| {
            statements.iter().filter_map(move |(origin, sql)| {
                check(sql, config)
                    .err()
                    .map(|e| format!("{origin} with {name}: {e}\n  {sql}"))
            })
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "{} of {} formats failed, the first {}:\n\n{}",
        failures.len(),
        statements.len() * configs.len(),
        MAX_REPORTED.min(failures.len()),
        failures
            .iter()
            .take(MAX_REPORTED)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n\n")
    );
}

#[test]
fn option_matrix_covers_every_layout_combination() {
    let configs = option_matrix();
    assert_eq!(configs.len(), 32);
    let pairs = configs
        .iter()
        .map(|(_, config)| {
            (
                config.indent_size,
                format!("{:?}", config.indent_style),
                config.line_width,
            )
        })
        .collect::<HashSet<_>>();
    assert_eq!(pairs.len(), 9);
}

#[test]
#[ignore = "slow in debug; CI runs it in release, see the module docs"]
fn fixtures_survive_every_option_combination() {
    assert_every_combination_survives(&fixture_statements());
}

#[test]
#[ignore = "slow in debug; CI runs it in release, see the module docs"]
fn regression_suite_survives_every_option_combination() {
    assert_every_combination_survives(&regression_statements());
}
