mod common;

use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgls_pretty_print::{
    CastStyle, ClauseBodyStyle, CommaStyle, FormatConfig, Layout, LogicalOperatorPlacement,
    emitter::EventEmitter,
    nodes::emit_statement,
    normalize::normalize_ast,
    renderer::{IndentStyle, KeywordCase, RenderConfig, Renderer},
};

/// Line widths to test - each test file is run at both widths
/// Minimum supported line width is 80 (60 is unrealistically narrow)
const LINE_WIDTHS: [usize; 2] = [80, 100];

/// Files with known platform-specific parsing differences (integer overflow handling)
/// These tests pass on Linux/macOS but fail on Windows due to pg_query library differences
#[cfg(target_os = "windows")]
const WINDOWS_SKIP_FILES: &[&str] = &[
    "combocid",
    "hash_index",
    "limit",
    "numerology",
    "partition_prune",
    "plpgsql",
    "portals_p2",
    "rangefuncs",
    "subselect",
    "tidscan",
];

/// A fixture may open with `-- pgls-format: key=value, key=value` to declare the configuration it
/// must be rendered with. Keeping it in the fixture rather than in the harness is what lets an
/// option that is off by default own its own test data.
///
/// It returns a `FormatConfig`, not a `RenderConfig`: some options decide which tokens exist and
/// are therefore read by the emitter, so the fixture has to reach both sides of the pipeline.
fn parse_fixture(content: &str) -> (FormatConfig, Option<usize>, String) {
    const HEADER: &str = "-- pgls-format:";

    let mut config = FormatConfig::default();
    let mut explicit_width = None;

    let Some(rest) = content.strip_prefix(HEADER) else {
        return (config, None, content.to_string());
    };

    let (header, sql) = match rest.split_once('\n') {
        Some((header, sql)) => (header, sql),
        None => (rest, ""),
    };

    for entry in header.split(',') {
        let Some((key, value)) = entry.split_once('=') else {
            panic!("malformed pgls-format entry: {entry}");
        };

        match (key.trim(), value.trim()) {
            ("lineWidth", value) => {
                let width = value.parse().expect("lineWidth must be a number");
                config.line_width = width;
                explicit_width = Some(width);
            }
            ("indentSize", value) => {
                config.indent_size = value.parse().expect("indentSize must be a number");
            }
            ("indentStyle", "tabs") => config.indent_style = IndentStyle::Tabs,
            ("indentStyle", "spaces") => config.indent_style = IndentStyle::Spaces,
            ("keywordCase", "upper") => config.keyword_case = KeywordCase::Upper,
            ("keywordCase", "lower") => config.keyword_case = KeywordCase::Lower,
            ("constantCase", "upper") => config.constant_case = KeywordCase::Upper,
            ("constantCase", "lower") => config.constant_case = KeywordCase::Lower,
            ("typeCase", "upper") => config.type_case = KeywordCase::Upper,
            ("typeCase", "lower") => config.type_case = KeywordCase::Lower,
            ("commaStyle", "leading") => config.comma_style = CommaStyle::Leading,
            ("commaStyle", "trailing") => config.comma_style = CommaStyle::Trailing,
            ("logicalOperatorPlacement", "leading") => {
                config.logical_operator_placement = LogicalOperatorPlacement::Leading;
            }
            ("logicalOperatorPlacement", "trailing") => {
                config.logical_operator_placement = LogicalOperatorPlacement::Trailing;
            }
            ("layout", "expanded") => config.layout = Layout::Expanded,
            ("layout", "fit") => config.layout = Layout::Fit,
            ("castStyle", "operator") => config.cast_style = CastStyle::Operator,
            ("castStyle", "cast") => config.cast_style = CastStyle::Cast,
            ("clauseBodyStyle", "break") => config.clause_body_style = ClauseBodyStyle::Break,
            ("clauseBodyStyle", "compact") => {
                config.clause_body_style = ClauseBodyStyle::Compact;
            }
            ("isolateSemicolon", value) => {
                config.isolate_semicolon = value
                    .parse()
                    .expect("isolateSemicolon must be true or false");
            }
            ("functionArgumentGroups", value) => {
                for entry in value.split('|') {
                    let Some((name, size)) = entry.split_once(':') else {
                        panic!("functionArgumentGroups entries must use name:size");
                    };
                    config.function_argument_groups.insert(
                        name.trim().to_lowercase(),
                        size.trim()
                            .parse()
                            .expect("functionArgumentGroups size must be a number"),
                    );
                }
            }
            (key, value) => panic!("unknown pgls-format entry: {key}={value}"),
        }
    }

    (config, explicit_width, sql.to_string())
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/single/",
    glob: "*.sql",
)]
fn test_single(fixture: Fixture<&str>) {
    let (fixture_config, explicit_width, content) = parse_fixture(fixture.content());

    println!("Original content:\n{content}");

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let base_test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    let widths: Vec<usize> = match explicit_width {
        Some(width) => vec![width],
        None => LINE_WIDTHS.to_vec(),
    };

    for max_line_length in widths {
        let test_name = format!("{base_test_name}_{max_line_length}");

        let parsed = pgls_query::parse(&content).expect("Failed to parse SQL");
        let mut ast = parsed.into_root().expect("No root node found");

        println!("Parsed AST: {ast:#?}");

        // The emitter gets the fixture config, not the default one: an option that decides which
        // tokens exist is read here, before the renderer ever sees the events.
        let mut emitter = EventEmitter::new(fixture_config.clone());
        emit_statement(&mut emitter, &ast);

        let mut output = String::new();
        let config = RenderConfig {
            max_line_length,
            ..RenderConfig::from(fixture_config.clone())
        };
        let mut renderer = Renderer::new(&mut output, config);
        renderer.render(emitter.events).expect("Failed to render");

        println!("Formatted content (width={max_line_length}):\n{output}");

        assert_line_lengths(&output, max_line_length, fixture_config.indent_size);
        common::assert_clean_whitespace(&output);
        common::assert_no_empty_statements(&output);

        let parsed_output = pgls_query::parse(&output).expect("Failed to parse SQL");
        let mut parsed_ast = parsed_output.into_root().expect("No root node found");

        normalize_ast(&mut parsed_ast);
        normalize_ast(&mut ast);

        assert_eq!(ast, parsed_ast);

        with_settings!({
            omit_expression => true,
            input_file => input_file,
            snapshot_path => "snapshots/single",
        }, {
            assert_snapshot!(test_name, output);
        });
    }
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/multi/",
    glob: "*.sql",
)]
fn test_multi(fixture: Fixture<&str>) {
    let absolute_fixture_path = Utf8Path::new(fixture.path());

    // Skip known problematic files on Windows due to platform-specific parser differences
    #[cfg(target_os = "windows")]
    {
        let file_stem = absolute_fixture_path
            .file_name()
            .and_then(|x| x.strip_suffix(".sql"))
            .unwrap_or("");
        if WINDOWS_SKIP_FILES.contains(&file_stem) {
            eprintln!(
                "Skipping {file_stem} on Windows due to known platform-specific parsing differences"
            );
            return;
        }
    }

    let (fixture_config, explicit_width, content) = parse_fixture(fixture.content());
    let input_file = absolute_fixture_path;
    let base_test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    let widths: Vec<usize> = match explicit_width {
        Some(width) => vec![width],
        None => LINE_WIDTHS.to_vec(),
    };

    for max_line_length in widths {
        let test_name = format!("{base_test_name}_{max_line_length}");

        // Split the content into statements
        let split_result = pgls_statement_splitter::split(&content);
        let mut formatted_statements = Vec::new();

        for range in &split_result.ranges {
            let statement = &content[usize::from(range.start())..usize::from(range.end())];
            let trimmed = statement.trim();

            if trimmed.is_empty() {
                continue;
            }

            let parsed = pgls_query::parse(trimmed).expect("Failed to parse SQL");
            let mut ast = parsed.into_root().expect("No root node found");

            println!("Parsed AST: {ast:#?}");

            // The emitter gets the fixture config, not the default one: an option that decides
            // which tokens exist is read here, before the renderer ever sees the events.
            let mut emitter = EventEmitter::new(fixture_config.clone());
            emit_statement(&mut emitter, &ast);

            let mut output = String::new();
            let config = RenderConfig {
                max_line_length,
                ..RenderConfig::from(fixture_config.clone())
            };
            let mut renderer = Renderer::new(&mut output, config);
            renderer.render(emitter.events).expect("Failed to render");

            // Verify line length
            assert_line_lengths(&output, max_line_length, fixture_config.indent_size);
            common::assert_clean_whitespace(&output);
            common::assert_no_empty_statements(&output);

            // Verify AST equality
            let parsed_output = pgls_query::parse(&output).unwrap_or_else(|e| {
                eprintln!("Failed to parse formatted SQL. Error: {e:?}");
                eprintln!("Statement index: {}", range.start());
                eprintln!("Formatted SQL:\n{output}");
                panic!("Failed to parse formatted SQL: {e:?}");
            });
            let mut parsed_ast = parsed_output.into_root().expect("No root node found");

            normalize_ast(&mut parsed_ast);
            normalize_ast(&mut ast);

            assert_eq!(ast, parsed_ast);

            formatted_statements.push(output);
        }

        // Join all formatted statements with double newline
        let final_output = formatted_statements.join("\n\n");

        println!("Formatted multi-statement content (width={max_line_length}):\n{final_output}");

        with_settings!({
            omit_expression => true,
            input_file => input_file,
            snapshot_path => "snapshots/multi",
        }, {
            assert_snapshot!(test_name, final_output);
        });
    }
}

fn assert_line_lengths(output: &str, max_line_length: usize, tab_width: usize) {
    let problems = common::width_problems(output, max_line_length, tab_width);
    assert!(
        problems.is_empty(),
        "lines past the width:\n{}\n\n{output}",
        problems.join("\n")
    );
}
