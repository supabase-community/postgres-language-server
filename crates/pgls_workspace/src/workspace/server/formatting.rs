//! Formatting of a document. Each statement is printed on its own, and the text between the
//! statements is kept, since the parser drops comments.

use std::{collections::HashMap, ops::Range, sync::Arc};

use pgls_diagnostics::{DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_pretty_print::FormatConfig;
use pgls_text_size::TextRange;

use super::{
    StatementId,
    document::{Document, FormatStatementMapper},
    sql_function,
};
use crate::{
    WorkspaceError,
    features::format::{PullFormattingResult, StatementFormatResult},
    settings::FormatterSettings,
};

fn format_config(settings: &FormatterSettings) -> FormatConfig {
    FormatConfig {
        line_width: settings.line_width as usize,
        indent_size: settings.indent_size as usize,
        indent_style: settings.indent_style.into(),
        keyword_case: settings.keyword_case.into(),
        constant_case: settings.constant_case.into(),
        type_case: settings.type_case.into(),
        comma_style: settings.comma_style.into(),
        logical_operator_placement: settings.logical_operator_placement.into(),
        layout: settings.layout.into(),
        values_row_layout: settings.values_row_layout.into(),
        cast_style: settings.cast_style.into(),
        clause_body_style: settings.clause_body_style.into(),
        isolate_semicolon: settings.isolate_semicolon,
        function_argument_groups: settings
            .function_argument_groups
            .iter()
            .map(|(name, size)| (name.to_lowercase(), usize::from(size.get())))
            .collect(),
    }
}

/// Formats the statements of `doc`, or only those that intersect `range`. `path` is the path of
/// the document in diagnostics.
pub(super) fn format_document(
    doc: &Document,
    settings: &FormatterSettings,
    range: Option<TextRange>,
    path: &str,
) -> PullFormattingResult {
    let config = format_config(settings);

    let mut diagnostics = Vec::new();
    let mut statements = Vec::new();
    let mut formatted_output = String::new();
    // The range of each statement in the source and in the output, and whether it parses.
    let mut spans: Vec<StatementSpan> = Vec::new();

    let format_candidates: Vec<_> = doc.iter(FormatStatementMapper).collect();
    let mut formatted_sql_fn_bodies = HashMap::new();

    // Only format SQL function bodies if skip_fn_bodies is false. A body with several
    // statements is left as written.
    if !settings.skip_fn_bodies {
        let mut statements_per_body: HashMap<StatementId, usize> = HashMap::new();
        for (id, ..) in &format_candidates {
            if let Some(parent_id) = id.parent() {
                *statements_per_body.entry(parent_id).or_default() += 1;
            }
        }
        for (id, _stmt_range, text, ast_result) in &format_candidates {
            if !id.is_child() {
                continue;
            }

            let Some(parent_id) = id.parent() else {
                continue;
            };

            if statements_per_body.get(&parent_id) != Some(&1) {
                continue;
            }

            let Ok(ast) = ast_result else {
                continue;
            };

            let Ok(result) = pgls_pretty_print::format_statement(ast, text, &config) else {
                continue;
            };

            let Some(formatted) = restore_named_parameters(text, &result.formatted, ast) else {
                continue;
            };

            formatted_sql_fn_bodies.insert(parent_id, formatted);
        }
    }

    // Reconstruct the file by replacing each statement's source span with its
    // formatted output while copying the gaps between statements (whitespace and
    // standalone comments) verbatim. libpg_query drops comments from the AST, so
    // anything that lives outside a statement's range would otherwise be lost.
    let source = doc.get_document_content();
    let mut last_end = 0usize;

    // Formatting a range leaves everything outside its statements as written.
    let collapses_blank_lines = range.is_none();

    for (id, stmt_range, text, ast_result) in format_candidates {
        if id.is_child() {
            continue;
        }

        let stmt_start = usize::from(stmt_range.start());
        let stmt_end = usize::from(stmt_range.end());

        // Preserve the standalone comments that precede this statement, with at most one
        // blank line between statements and none before the first.
        let gap = &source[last_end..stmt_start];
        if collapses_blank_lines {
            let gap = collapse_blank_lines(gap);
            formatted_output.push_str(if last_end == 0 {
                gap.trim_start()
            } else {
                &gap
            });
        } else {
            formatted_output.push_str(gap);
        }
        last_end = stmt_end;

        let output_start = formatted_output.len();
        let parses = ast_result.is_ok();

        // Leave statements outside the requested range untouched.
        if let Some(filter_range) = range
            && stmt_range.intersect(filter_range).is_none()
        {
            formatted_output.push_str(&text);
            spans.push(StatementSpan {
                source: stmt_start..stmt_end,
                output: output_start..formatted_output.len(),
                parses,
            });
            continue;
        }

        match ast_result {
            Ok(ast) => {
                let mut ast = ast;
                if let Some(formatted_sql_fn_body) = formatted_sql_fn_bodies.get(&id) {
                    // Copies the tree only for the functions whose body is replaced.
                    sql_function::set_formatted_sql_fn_body(
                        Arc::make_mut(&mut ast),
                        formatted_sql_fn_body,
                    );
                }

                match pgls_pretty_print::format_statement(&ast, &text, &config) {
                    Ok(result) => match restore_named_parameters(&text, &result.formatted, &ast) {
                        Some(formatted) => {
                            if text != formatted {
                                statements.push(StatementFormatResult {
                                    original: text.clone(),
                                    formatted: formatted.clone(),
                                    range: stmt_range,
                                });
                            }
                            formatted_output.push_str(&formatted);
                        }
                        None => {
                            diagnostics.push(SDiagnostic::new(
                                pgls_diagnostics::Error::from(WorkspaceError::format_error(
                                    "Named parameters could not be restored after \
                                             formatting, the statement was left untouched"
                                        .to_string(),
                                ))
                                .with_file_path(path)
                                .with_file_span(stmt_range),
                            ));

                            formatted_output.push_str(&text);
                        }
                    },
                    Err(err) => {
                        diagnostics.push(SDiagnostic::new(
                            pgls_diagnostics::Error::from(WorkspaceError::format_error(
                                err.to_string(),
                            ))
                            .with_file_path(path)
                            .with_file_span(stmt_range),
                        ));

                        formatted_output.push_str(&text);
                    }
                }
            }
            Err(syntax_err) => {
                diagnostics.push(SDiagnostic::new(
                    pgls_diagnostics::Error::from(syntax_err).with_file_path(path),
                ));

                formatted_output.push_str(&text);
            }
        }

        spans.push(StatementSpan {
            source: stmt_start..stmt_end,
            output: output_start..formatted_output.len(),
            parses,
        });
    }

    // Preserve any trailing text or comments after the last statement, without blank lines
    // at the end of the file but with its final line break.
    let rest = &source[last_end..];
    if collapses_blank_lines {
        let rest = collapse_blank_lines(rest);
        let rest = if last_end == 0 {
            rest.trim()
        } else {
            rest.trim_end()
        };
        formatted_output.push_str(rest);
        if source.ends_with('\n') && !formatted_output.is_empty() {
            formatted_output.push('\n');
        }
    } else {
        formatted_output.push_str(rest);
    }

    if let Err(reason) = check_around_unparsable(source, &formatted_output, &spans) {
        diagnostics.push(SDiagnostic::new(
            pgls_diagnostics::Error::from(WorkspaceError::format_error(format!(
                "The formatted file would not mean the same as the original ({reason}), the file \
                 was left untouched"
            )))
            .with_file_path(path),
        ));

        return PullFormattingResult {
            original: source.to_string(),
            formatted: source.to_string(),
            statements: Vec::new(),
            diagnostics,
        };
    }

    PullFormattingResult {
        original: doc.get_document_content().to_string(),
        formatted: formatted_output,
        statements,
        diagnostics,
    }
}

/// Where a statement of the document is in the source and in the formatted output.
struct StatementSpan {
    source: Range<usize>,
    output: Range<usize>,
    /// Whether the statement parses on its own. One that does not is kept as written.
    parses: bool,
}

/// Each statement was checked on its own, but that relies on where the statement splitter put the
/// boundaries. A wrong boundary cuts a valid statement into fragments, at least one of which does
/// not parse, and the others may format into something else, e.g. the `END` of a function body
/// into `COMMIT` (#1142).
///
/// So the text from the statement before the first one that does not parse to the statement after
/// the last one is parsed as a whole, before and after formatting, independently of the splitter.
///
/// Costs nothing for a file whose statements all parse. Otherwise up to four parses of that text,
/// about 40 ms per 1000 statements each, so 3 s for 20k statements with errors at both ends, on
/// top of about 2.4 s of formatting them.
fn check_around_unparsable(
    source: &str,
    formatted: &str,
    spans: &[StatementSpan],
) -> Result<(), String> {
    let (Some(first), Some(last)) = (
        spans.iter().position(|span| !span.parses),
        spans.iter().rposition(|span| !span.parses),
    ) else {
        return Ok(());
    };

    let Some(around) = spans.get(first.saturating_sub(1)..spans.len().min(last + 2)) else {
        return Ok(());
    };
    let (Some(from), Some(to)) = (around.first(), around.last()) else {
        return Ok(());
    };
    let source_range = from.source.start..to.source.end;
    let output_range = from.output.start..to.output.end;
    let (Some(source), Some(formatted)) = (
        source.get(source_range.clone()),
        formatted.get(output_range.clone()),
    ) else {
        debug_assert!(false, "statement ranges at no character boundary");
        return Err("its statements could not be told apart".to_string());
    };

    if source == formatted {
        return Ok(());
    }

    let unparsable: Vec<_> = around
        .iter()
        .filter(|span| !span.parses)
        .map(|span| {
            (
                span.source.start - source_range.start..span.source.end - source_range.start,
                span.output.start - output_range.start..span.output.end - output_range.start,
            )
        })
        .collect();

    check_unchanged(source, formatted, &unparsable)
}

/// Checks that `formatted` parses as a whole into the same statements as `source`.
///
/// `unparsable` holds the ranges of the statements that were kept as written because they do not
/// parse, in `source` and in `formatted`. When `source` does not parse as a whole, the check is
/// repeated with them blanked out, so a syntax error or a psql meta-command in one statement does
/// not turn it off for the rest. When neither parses, nothing can be checked.
fn check_unchanged(
    source: &str,
    formatted: &str,
    unparsable: &[(Range<usize>, Range<usize>)],
) -> Result<(), String> {
    let (expected, formatted) = match parse_file(source) {
        Some(expected) => (expected, formatted.to_string()),
        None => {
            let Some(expected) =
                blank_out(source, unparsable.iter().map(|r| &r.0)).and_then(|s| parse_file(&s))
            else {
                return Ok(());
            };
            let Some(formatted) = blank_out(formatted, unparsable.iter().map(|r| &r.1)) else {
                return Err("its statements could not be told apart".to_string());
            };
            (expected, formatted)
        }
    };

    let Some(actual) = parse_file(&formatted) else {
        return Err("it has a syntax error".to_string());
    };

    if actual.len() != expected.len() {
        return Err(format!(
            "it has {} statements instead of {}",
            actual.len(),
            expected.len()
        ));
    }

    match actual.iter().zip(&expected).position(|(a, e)| a != e) {
        Some(idx) => Err(format!("statement {} changed", idx + 1)),
        None => Ok(()),
    }
}

/// The normalized statements of `text`, or `None` if it does not parse. The bodies of SQL
/// functions are emptied, since they are formatted and checked on their own.
fn parse_file(text: &str) -> Option<Vec<pgls_query::NodeEnum>> {
    let parsed = pgls_query::parse(&pgls_lexer::convert_to_positional_params(text)).ok()?;
    Some(
        parsed
            .protobuf
            .stmts
            .into_iter()
            .filter_map(|raw| raw.stmt.and_then(|stmt| stmt.node))
            .map(|mut stmt| {
                sql_function::clear_sql_fn_body(&mut stmt);
                pgls_pretty_print::normalize_ast(&mut stmt);
                stmt
            })
            .collect(),
    )
}

/// `text` with the bytes of `ranges` replaced by spaces, so the rest keeps its offsets. `None` if a
/// range does not start and end at a character boundary.
fn blank_out<'a>(text: &str, ranges: impl Iterator<Item = &'a Range<usize>>) -> Option<String> {
    let mut bytes = text.as_bytes().to_vec();
    for range in ranges {
        debug_assert!(
            range.end <= bytes.len(),
            "range {range:?} is outside of the text"
        );
        if let Some(blanked) = bytes.get_mut(range.clone()) {
            blanked.fill(b' ');
        }
    }
    String::from_utf8(bytes).ok()
}

/// Keeps at most one blank line in `text`, which only holds whitespace and comments.
fn collapse_blank_lines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    // Line breaks since the last text that is not whitespace. Two make a blank line.
    let mut line_breaks = 0;
    for line in text.split_inclusive('\n') {
        let is_blank = line.trim().is_empty();
        let ends_line = line.ends_with('\n');
        if is_blank && ends_line {
            line_breaks += 1;
            if line_breaks <= 2 {
                result.push('\n');
            }
            continue;
        }

        result.push_str(line);
        if !is_blank {
            line_breaks = usize::from(ends_line);
        }
    }
    result
}

/// Put psql named parameters back where the formatter printed their placeholders.
///
/// The parser only ever sees placeholders, so the printed statement carries `$1` or a bare
/// identifier instead of `:'agen_code'` or `:raw_data`. Substitution happens on the token stream
/// so that a `$1` inside a string literal is left alone, and the result is proven by converting it
/// back and comparing the normalized ASTs: a statement that does not survive that round trip is
/// reported as unrestorable and kept as it was written.
fn restore_named_parameters(
    original: &str,
    formatted: &str,
    original_ast: &pgls_query::NodeEnum,
) -> Option<String> {
    use pgls_lexer::{SyntaxKind, lex};

    let conversion = pgls_lexer::convert_to_positional_params_with_metadata(original);

    if conversion.identifier_replacements.is_empty() && conversion.value_replacements.is_empty() {
        return Some(formatted.to_string());
    }

    if !conversion.restorable {
        return None;
    }

    let lexed = lex(formatted);
    let mut restored = String::with_capacity(formatted.len());

    for (idx, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::EOF {
            break;
        }

        let text = lexed.text(idx);

        let replacement = match kind {
            SyntaxKind::IDENT => conversion.identifier_replacements.get(text),
            SyntaxKind::POSITIONAL_PARAM => text
                .trim_start_matches('$')
                .parse::<usize>()
                .ok()
                .and_then(|index| conversion.value_replacements.get(&index)),
            _ => None,
        };

        match replacement {
            Some(source_text) => restored.push_str(source_text),
            None => restored.push_str(text),
        }
    }

    let reparsed = pgls_query::parse(&pgls_lexer::convert_to_positional_params(&restored)).ok()?;
    let mut restored_ast = reparsed.into_root()?;
    let mut expected_ast = original_ast.clone();
    pgls_pretty_print::normalize_ast(&mut restored_ast);
    pgls_pretty_print::normalize_ast(&mut expected_ast);

    if restored_ast == expected_ast {
        Some(restored)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{StatementSpan, check_around_unparsable, check_unchanged};

    const BEGIN_ATOMIC: &str = "CREATE FUNCTION f() RETURNS integer LANGUAGE sql
BEGIN ATOMIC
  SELECT CASE WHEN true THEN 1 ELSE 2 END;
END;
";

    #[test]
    fn rejects_a_closing_end_formatted_as_commit() {
        // What a wrong statement boundary in the body made of it, see #1142.
        let formatted = BEGIN_ATOMIC.replace("END;\n", "commit;\n");
        let body_end = BEGIN_ATOMIC.find("END;\n").unwrap();
        let unparsable = [(0..body_end, 0..body_end)];

        assert_eq!(
            check_unchanged(BEGIN_ATOMIC, &formatted, &unparsable),
            Err("it has a syntax error".to_string())
        );
    }

    #[test]
    fn checks_the_statements_around_one_that_does_not_parse() {
        // How the splitter cut the function before #1142 was fixed.
        let body_end = BEGIN_ATOMIC.find("END;\n").unwrap();
        let source = format!("select 1;\n{BEGIN_ATOMIC}");
        let formatted = format!("select 1;\n{}", BEGIN_ATOMIC.replace("END;\n", "commit;\n"));
        let spans = |parses: bool| {
            [
                StatementSpan {
                    source: 0..9,
                    output: 0..9,
                    parses: true,
                },
                StatementSpan {
                    source: 10..10 + body_end,
                    output: 10..10 + body_end,
                    parses,
                },
                StatementSpan {
                    source: 10 + body_end..source.len() - 1,
                    output: 10 + body_end..formatted.len() - 1,
                    parses: true,
                },
            ]
        };

        assert_eq!(
            check_around_unparsable(&source, &formatted, &spans(false)),
            Err("it has a syntax error".to_string())
        );
        // Statements that all parse are not checked again.
        assert_eq!(
            check_around_unparsable(&source, &formatted, &spans(true)),
            Ok(())
        );
    }

    #[test]
    fn accepts_a_reformatted_file() {
        let source =
            "SELECT a,b FROM t;\nCREATE FUNCTION f() RETURNS int AS 'SELECT 1' LANGUAGE sql;";
        let formatted =
            "select a, b from t;\ncreate function f() returns int as '\nselect 1;\n' language sql;";

        assert_eq!(check_unchanged(source, formatted, &[]), Ok(()));
        assert_eq!(check_unchanged(BEGIN_ATOMIC, BEGIN_ATOMIC, &[]), Ok(()));
    }

    #[test]
    fn rejects_a_changed_statement() {
        assert_eq!(
            check_unchanged("select 1;\nselect 2;", "select 1;\nselect 3;", &[]),
            Err("statement 2 changed".to_string())
        );
        assert_eq!(
            check_unchanged("select 1;", "select 1;\ncommit;", &[]),
            Err("it has 2 statements instead of 1".to_string())
        );
    }

    #[test]
    fn checks_around_unparsable_statements() {
        let source = "SELECT a,b FROM t;\nselect from where;\n\\set x 1\nSELECT 2;\n";
        let error = source.find("select from").unwrap()..source.find("SELECT 2").unwrap();

        let formatted = "select a, b from t;\nselect from where;\n\\set x 1\nselect 2;\n";
        let output_error =
            formatted.find("select from").unwrap()..formatted.find("select 2").unwrap();
        assert_eq!(
            check_unchanged(source, formatted, &[(error.clone(), output_error)]),
            Ok(())
        );

        let formatted = "select a, b from t;\nselect from where;\n\\set x 1\ncommit;\n";
        let output_error =
            formatted.find("select from").unwrap()..formatted.find("commit").unwrap();
        assert_eq!(
            check_unchanged(source, formatted, &[(error, output_error)]),
            Err("statement 2 changed".to_string())
        );
    }
}
