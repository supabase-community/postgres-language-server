//! Formatting of a document. Each statement is printed on its own, and the text between the
//! statements is kept, since the parser drops comments.

use std::{ops::Range, sync::Arc};

use pgls_analyser::STOP_CHECK_INTERVAL;
use pgls_diagnostics::{DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_lexer::SyntaxKind;
use pgls_pretty_print::FormatConfig;
use pgls_query_ext::{diagnostics::SyntaxDiagnostic, function_body::sql_function_body};
use pgls_text_size::{TextRange, TextSize};

use super::{
    document::{Document, FormatStatementMapper},
    document_cache::parse_statement,
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
///
/// `None` if `should_stop` stopped it, e.g. because the document changed or the request was
/// cancelled. It is checked every [`STOP_CHECK_INTERVAL`] statements that are parsed or
/// formatted.
pub(super) fn format_document(
    doc: &Document,
    settings: &FormatterSettings,
    range: Option<TextRange>,
    path: &str,
    should_stop: &dyn Fn() -> bool,
) -> Option<PullFormattingResult> {
    let config = format_config(settings);
    let mut visited = 0usize;
    let mut stops = || {
        visited += 1;
        visited.is_multiple_of(STOP_CHECK_INTERVAL) && should_stop()
    };

    let mut diagnostics = Vec::new();
    let mut statements = Vec::new();
    let mut formatted_output = String::new();

    let mut split_statements: Vec<FormatStatement> = Vec::new();
    for (id, stmt_range, text, ast) in doc.iter(FormatStatementMapper) {
        if stops() {
            return None;
        }
        // The statements of a SQL function body follow the function.
        if id.is_child() {
            if let Some(function) = split_statements.last_mut() {
                function.body.push((text, ast));
            }
            continue;
        }
        split_statements.push(FormatStatement {
            range: stmt_range,
            text,
            ast,
            body: Vec::new(),
        });
    }

    let source = doc.get_document_content();

    // The splitter guesses the boundaries of the statements, so that it can deal with SQL that is
    // being written. When the file parses as a whole, the parser knows them.
    let format_statements = match parser_statement_ranges(doc.text(), source) {
        Some(ranges) => {
            // Most statements are those of the splitter, whose trees are cached.
            let mut split_statements = split_statements.into_iter().peekable();
            let mut statements = Vec::with_capacity(ranges.len());
            for (stmt_range, text) in ranges {
                if stops() {
                    return None;
                }
                while split_statements
                    .next_if(|stmt| stmt.range.start() < stmt_range.start())
                    .is_some()
                {}
                statements.push(
                    split_statements
                        .next_if(|stmt| stmt.range == stmt_range)
                        .unwrap_or_else(|| FormatStatement::parse(stmt_range, text)),
                );
            }
            statements
        }
        None => split_statements,
    };

    // Reconstruct the file by replacing each statement's source span with its
    // formatted output while copying the gaps between statements (whitespace and
    // standalone comments) verbatim. libpg_query drops comments from the AST, so
    // anything that lives outside a statement's range would otherwise be lost.
    let mut last_end = 0usize;

    // Formatting a range leaves everything outside its statements as written.
    let collapses_blank_lines = range.is_none();

    for FormatStatement {
        range: stmt_range,
        text,
        ast: ast_result,
        body,
    } in format_statements
    {
        if stops() {
            return None;
        }

        let stmt_start = usize::from(stmt_range.start());
        let stmt_end = usize::from(stmt_range.end());
        debug_assert!(
            last_end <= stmt_start
                && source.is_char_boundary(stmt_start)
                && source.is_char_boundary(stmt_end),
            "statement {stmt_range:?} overlaps the one before it or isn't at character boundaries"
        );

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

        // Leave statements outside the requested range untouched, and those with `ENVSUB`
        // expansions, whose text holds placeholders.
        if range.is_some_and(|filter_range| stmt_range.intersect(filter_range).is_none())
            || doc.touches_placeholder(stmt_range)
        {
            formatted_output.push_str(&source[stmt_start..stmt_end]);
            continue;
        }

        match ast_result {
            Ok(ast) => {
                let mut ast = ast;
                // A SQL function body with several statements is left as written.
                if let [(body_text, Ok(body_ast))] = body.as_slice()
                    && !settings.skip_fn_bodies
                    && let Some(formatted_sql_fn_body) =
                        format_sql_fn_body(body_text, body_ast, &config)
                {
                    // Copies the tree only for the functions whose body is replaced.
                    sql_function::set_formatted_sql_fn_body(
                        Arc::make_mut(&mut ast),
                        &formatted_sql_fn_body,
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

    Some(PullFormattingResult {
        original: doc.get_document_content().to_string(),
        formatted: formatted_output,
        statements,
        diagnostics,
    })
}

/// A top-level statement to format.
struct FormatStatement {
    range: TextRange,
    text: String,
    ast: Result<Arc<pgls_query::NodeEnum>, SyntaxDiagnostic>,
    /// The text and tree of each statement of its SQL function body.
    body: Vec<(String, Result<Arc<pgls_query::NodeEnum>, SyntaxDiagnostic>)>,
}

impl FormatStatement {
    /// Parses the statement `text` at `range`, which the splitter does not have, so its tree
    /// isn't cached.
    fn parse(range: TextRange, text: &str) -> Self {
        let ast = parse_statement(text).map_err(|diag| diag.span(range));
        let body = match &ast {
            Ok(ast) => sql_function_body(ast, text)
                .into_iter()
                .map(|embedded| {
                    let ast = parse_statement(&embedded.sql);
                    (embedded.sql, ast)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        Self {
            range,
            text: text.to_string(),
            ast,
            body,
        }
    }
}

/// The formatted body of a SQL function with the single statement `text`, or `None` if it can't
/// be formatted.
fn format_sql_fn_body(
    text: &str,
    ast: &pgls_query::NodeEnum,
    config: &FormatConfig,
) -> Option<String> {
    let result = pgls_pretty_print::format_statement(ast, text, config).ok()?;
    restore_named_parameters(text, &result.formatted, ast)
}

/// The range and text of each top-level statement of `text`, as the parser finds them, or `None`
/// if `text` does not parse as a whole, e.g. because of a syntax error or a psql meta-command.
/// The ranges are those the splitter would have: from the first token of the statement to its
/// `;`. `source` is the content of the document, of which `text` replaces the placeholders of
/// the migration tool.
///
/// The splitter can cut a statement in the wrong place, e.g. `CREATE SCHEMA s CREATE TABLE t (a
/// int);` before the second `CREATE`, and formatting the parts on their own would change what
/// the file means.
///
/// Costs one parse of the file, without building its tree: 45 ms for 20k statements in a release
/// build. The trees of the statements are those the document caches for the splitter's
/// statements; only the statements that the splitter cuts differently are parsed again.
fn parser_statement_ranges<'a>(text: &'a str, source: &str) -> Option<Vec<(TextRange, &'a str)>> {
    let converted = pgls_lexer::convert_to_positional_params(text);
    // Named parameters are replaced in place, so the offsets are those of `text`.
    debug_assert_eq!(converted.len(), text.len());
    if converted.len() != text.len() {
        return None;
    }
    let ranges = pgls_query::split_ranges_with_parser(&converted).ok()?;

    let mut statements: Vec<(TextRange, &str)> = Vec::with_capacity(ranges.len());
    for range in ranges {
        let range = statement_range(text, range)?;
        let stmt_range = Range::<usize>::from(range);
        // The text between the statements is copied from `source`.
        if !source.is_char_boundary(stmt_range.start) || !source.is_char_boundary(stmt_range.end) {
            debug_assert!(false, "statement {range:?} isn't at character boundaries");
            return None;
        }
        debug_assert!(
            statements
                .last()
                .is_none_or(|(last, _)| last.end() <= range.start()),
            "statement {range:?} overlaps the one before it"
        );
        statements.push((range, text.get(stmt_range)?));
    }
    Some(statements)
}

/// The range of the statement at `range` of `text`, as the parser reports it, with its `;`. The
/// last statement without a `;` ends at its last token, before the comments after it.
fn statement_range(text: &str, range: Range<usize>) -> Option<TextRange> {
    let stmt = text.get(range.clone())?;
    let end = if text.get(range.end..)?.starts_with(';') {
        range.end + 1
    } else {
        let lexed = pgls_lexer::lex(stmt);
        let last = (0..lexed.len()).rev().find(|&idx| {
            let kind = lexed.kind(idx);
            kind != SyntaxKind::EOF && !kind.is_trivia()
        })?;
        range.start + usize::from(lexed.range(last).end())
    };
    debug_assert!(text.is_char_boundary(end));
    Some(TextRange::new(
        TextSize::try_from(range.start).ok()?,
        TextSize::try_from(end).ok()?,
    ))
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
