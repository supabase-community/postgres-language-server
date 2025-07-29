use std::io;

use pgt_console::markup;
use pgt_diagnostics::{Advices, Diagnostic, LogCategory, MessageAndDescription, Severity, Visit};
use pgt_text_size::TextRange;

use crate::{PlpgSqlCheckIssue, PlpgSqlCheckResult};

/// A specialized diagnostic for plpgsql_check.
#[derive(Clone, Debug, Diagnostic)]
#[diagnostic(category = "plpgsql_check")]
pub struct PlPgSqlCheckDiagnostic {
    #[location(span)]
    pub span: Option<TextRange>,
    #[description]
    #[message]
    pub message: MessageAndDescription,
    #[advice]
    pub advices: PlPgSqlCheckAdvices,
    #[severity]
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub struct PlPgSqlCheckAdvices {
    pub code: Option<String>,
}

impl Advices for PlPgSqlCheckAdvices {
    fn record(&self, visitor: &mut dyn Visit) -> io::Result<()> {
        // Show the error code if available
        if let Some(code) = &self.code {
            visitor.record_log(
                LogCategory::Error,
                &markup! { "SQL State: " <Emphasis>{code}</Emphasis> },
            )?;
        }

        Ok(())
    }
}

/// Convert plpgsql_check results into diagnostics
pub fn create_diagnostics_from_check_result(
    result: &PlpgSqlCheckResult,
    fn_body: &str,
    offset: usize,
) -> Vec<PlPgSqlCheckDiagnostic> {
    result
        .issues
        .iter()
        .map(|issue| create_diagnostic_from_issue(issue, fn_body, offset))
        .collect()
}

fn create_diagnostic_from_issue(
    issue: &PlpgSqlCheckIssue,
    fn_body: &str,
    offset: usize,
) -> PlPgSqlCheckDiagnostic {
    let severity = match issue.level.as_str() {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        "notice" => Severity::Hint,
        _ => Severity::Information,
    };

    PlPgSqlCheckDiagnostic {
        message: issue.message.clone().into(),
        severity,
        span: resolve_span(issue, fn_body, offset),
        advices: PlPgSqlCheckAdvices {
            code: issue.sql_state.clone(),
        },
    }
}

fn resolve_span(issue: &PlpgSqlCheckIssue, fn_body: &str, offset: usize) -> Option<TextRange> {
    let stmt = issue.statement.as_ref()?;

    let line_number = stmt
        .line_number
        .parse::<usize>()
        .expect("Expected line number to be a valid usize");

    let text = &stmt.text;

    // calculate the offset to the target line
    let line_offset: usize = fn_body
        .lines()
        .take(line_number - 1)
        .map(|line| line.len() + 1) // +1 for newline
        .sum();

    // find the position within the target line
    let line = fn_body.lines().nth(line_number - 1)?;
    let start = line
        .to_lowercase()
        .find(&text.to_lowercase())
        .unwrap_or_else(|| {
            line.char_indices()
                .find_map(|(i, c)| if !c.is_whitespace() { Some(i) } else { None })
                .unwrap_or(0)
        });

    let stmt_offset = line_offset + start;

    if let Some(q) = &issue.query {
        // first find the query within the fn body *after* stmt_offset
        let query_start = fn_body[stmt_offset..]
            .to_lowercase()
            .find(&q.text.to_lowercase())
            .map(|pos| pos + stmt_offset);

        // the position is *within* the query text
        let pos = q
            .position
            .parse::<usize>()
            .expect("Expected query position to be a valid usize")
            - 1; // -1 because the position is 1-based

        let start = query_start? + pos;

        // the range of the diagnostics is the token that `pos` is on
        // Find the end of the current token by looking for whitespace or SQL delimiters
        let remaining = &fn_body[start..];
        let end = remaining
            .char_indices()
            .find(|(_, c)| {
                c.is_whitespace() || matches!(c, ',' | ';' | ')' | '(' | '=' | '<' | '>')
            })
            .map(|(i, c)| {
                if matches!(c, ';') {
                    i + 1 // include the semicolon
                } else {
                    i // just the token end
                }
            })
            .unwrap_or(remaining.len());

        return Some(TextRange::new(
            ((offset + start) as u32).into(),
            ((offset + start + end) as u32).into(),
        ));
    }

    // if no query is present, the end range covers
    // - if text is "IF" or "ELSIF", then until the next "THEN"
    // - TODO: check "LOOP", "CASE", "WHILE", "EXPECTION" and others
    // - else: until the next semicolon or end of line

    if text.to_uppercase() == "IF" || text.to_uppercase() == "ELSIF" {
        // Find the position of the next "THEN" after the statement
        let remaining = &fn_body[stmt_offset..];
        if let Some(then_pos) = remaining.to_uppercase().find("THEN") {
            let end = then_pos + "THEN".len();
            return Some(TextRange::new(
                ((offset + stmt_offset) as u32).into(),
                ((offset + stmt_offset + end) as u32).into(),
            ));
        }
    }

    // if no specific end is found, use the next semicolon or the end of the line
    let remaining = &fn_body[stmt_offset..];
    let end = remaining
        .char_indices()
        .find(|(_, c)| matches!(c, ';' | '\n' | '\r'))
        .map(|(i, c)| {
            if c == ';' {
                i + 1 // include the semicolon
            } else {
                i // just the end of the line
            }
        })
        .unwrap_or(remaining.len());

    Some(TextRange::new(
        ((offset + stmt_offset) as u32).into(),
        ((offset + stmt_offset + end) as u32).into(),
    ))
}
