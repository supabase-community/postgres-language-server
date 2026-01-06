use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgls_pretty_print::{
    emitter::EventEmitter,
    nodes::emit_node_enum,
    normalize::normalize_ast,
    renderer::{IndentStyle, RenderConfig, Renderer},
};

/// Line widths to test - each test file is run at both widths
/// Minimum supported line width is 80 (60 is unrealistically narrow)
const LINE_WIDTHS: [usize; 2] = [80, 100];

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringState {
    None,
    Single,
    Double,
    Dollar(Vec<char>),
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/single/",
    glob: "*.sql",
)]
fn test_single(fixture: Fixture<&str>) {
    let content = fixture.content();

    println!("Original content:\n{content}");

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let base_test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // Run test at each configured line width
    for &max_line_length in &LINE_WIDTHS {
        let test_name = format!("{base_test_name}_{max_line_length}");

        let parsed = pgls_query::parse(content).expect("Failed to parse SQL");
        let mut ast = parsed.into_root().expect("No root node found");

        println!("Parsed AST: {ast:#?}");

        let mut emitter = EventEmitter::new();
        emit_node_enum(&ast, &mut emitter);

        let mut output = String::new();
        let config = RenderConfig {
            max_line_length,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            ..Default::default()
        };
        let mut renderer = Renderer::new(&mut output, config);
        renderer.render(emitter.events).expect("Failed to render");

        println!("Formatted content (width={max_line_length}):\n{output}");

        assert_line_lengths(&output, max_line_length);

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
    let content = fixture.content();

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let base_test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // Run test at each configured line width
    for &max_line_length in &LINE_WIDTHS {
        let test_name = format!("{base_test_name}_{max_line_length}");

        // Split the content into statements
        let split_result = pgls_statement_splitter::split(content);
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

            let mut emitter = EventEmitter::new();
            emit_node_enum(&ast, &mut emitter);

            let mut output = String::new();
            let config = RenderConfig {
                max_line_length,
                indent_size: 2,
                indent_style: IndentStyle::Spaces,
                ..Default::default()
            };
            let mut renderer = Renderer::new(&mut output, config);
            renderer.render(emitter.events).expect("Failed to render");

            // Verify line length
            assert_line_lengths(&output, max_line_length);

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

fn assert_line_lengths(sql: &str, max_line_length: usize) {
    let mut state = StringState::None;

    for line in sql.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        let mut current_outside_run = 0usize;
        let mut max_outside_run = 0usize;
        let mut longest_token = 0usize;
        let mut current_token_len = 0usize;

        while i < chars.len() {
            match state.clone() {
                StringState::None => {
                    current_outside_run += 1;
                    if current_outside_run > max_outside_run {
                        max_outside_run = current_outside_run;
                    }

                    // Track token length (identifier-like sequences and numeric literals)
                    // Numeric literals can include digits, '.', '-', '+', 'e', 'E'
                    let is_token_char = chars[i].is_alphanumeric()
                        || chars[i] == '_'
                        || chars[i] == '.'
                        || ((chars[i] == '-' || chars[i] == '+')
                            && (current_token_len == 0
                                || (i > 0 && (chars[i - 1] == 'e' || chars[i - 1] == 'E'))));

                    if is_token_char {
                        current_token_len += 1;
                    } else {
                        if current_token_len > longest_token {
                            longest_token = current_token_len;
                        }
                        current_token_len = 0;
                    }

                    match chars[i] {
                        '\'' => {
                            state = StringState::Single;
                            current_outside_run = 0;
                            i += 1;
                        }
                        '"' => {
                            state = StringState::Double;
                            current_outside_run = 0;
                            i += 1;
                        }
                        '$' => {
                            if let Some((tag, len)) = parse_dollar_tag(&chars[i..]) {
                                state = StringState::Dollar(tag);
                                current_outside_run = 0;
                                i += len;
                            } else {
                                i += 1;
                            }
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }
                StringState::Single => {
                    if chars[i] == '\'' {
                        if i + 1 < chars.len() && chars[i + 1] == '\'' {
                            i += 2;
                        } else {
                            state = StringState::None;
                            current_outside_run = 0;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                StringState::Double => {
                    if chars[i] == '"' {
                        if i + 1 < chars.len() && chars[i + 1] == '"' {
                            i += 2;
                        } else {
                            state = StringState::None;
                            current_outside_run = 0;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                StringState::Dollar(tag) => {
                    if chars[i] == '$' && slice_starts_with(&chars[i..], &tag) {
                        state = StringState::None;
                        current_outside_run = 0;
                        i += tag.len();
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Check final token
        if current_token_len > longest_token {
            longest_token = current_token_len;
        }

        // If the longest token exceeds max length, the line can't be broken further
        // Allow lines where the excess is due to an unbreakable identifier/literal
        // Also allow some overhead for indentation, keywords, and punctuation (up to 20 chars)
        // This accounts for things like "CREATE TRIGGER " or "EXECUTE FUNCTION " prefixes
        let min_overhead = 20;
        if max_outside_run > max_line_length && longest_token + min_overhead <= max_line_length {
            panic!("Line exceeds max length of {max_line_length} outside literals: {line}");
        }
    }
}

fn parse_dollar_tag(chars: &[char]) -> Option<(Vec<char>, usize)> {
    if chars.is_empty() || chars[0] != '$' {
        return None;
    }

    let mut end = 1usize;
    while end < chars.len() {
        let c = chars[end];
        if c.is_ascii_alphanumeric() || c == '_' {
            end += 1;
        } else {
            break;
        }
    }

    if end < chars.len() && chars[end] == '$' {
        let mut tag = Vec::with_capacity(end + 1);
        tag.extend_from_slice(&chars[..=end]);
        Some((tag, end + 1))
    } else {
        None
    }
}

fn slice_starts_with(haystack: &[char], needle: &[char]) -> bool {
    haystack.len() >= needle.len() && haystack[..needle.len()] == needle[..]
}
