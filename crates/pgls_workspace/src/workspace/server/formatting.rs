//! Formatting of a document. Each statement is printed on its own, and the text between the
//! statements is kept, since the parser drops comments.

use std::{collections::HashMap, sync::Arc};

use pgls_diagnostics::{DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_pretty_print::FormatConfig;
use pgls_text_size::TextRange;

use super::{
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
    let mut diagnostics = Vec::new();
    let mut statements = Vec::new();
    let mut formatted_output = String::new();

    let config = format_config(settings);
    let format_candidates: Vec<_> = doc.iter(FormatStatementMapper).collect();
    let mut formatted_sql_fn_bodies = HashMap::new();

    // Only format SQL function bodies if skip_fn_bodies is false
    if !settings.skip_fn_bodies {
        for (id, _stmt_range, text, ast_result) in &format_candidates {
            if !id.is_child() {
                continue;
            }

            let Some(parent_id) = id.parent() else {
                continue;
            };

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

        // Leave statements outside the requested range untouched.
        if let Some(filter_range) = range
            && stmt_range.intersect(filter_range).is_none()
        {
            formatted_output.push_str(&text);
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

    PullFormattingResult {
        original: doc.get_document_content().to_string(),
        formatted: formatted_output,
        statements,
        diagnostics,
    }
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
