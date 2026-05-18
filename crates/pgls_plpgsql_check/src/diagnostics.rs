use std::io;

use pgls_console::markup;
use pgls_diagnostics::{Advices, Diagnostic, LogCategory, MessageAndDescription, Severity, Visit};
use pgls_text_size::TextRange;

use crate::{PlpgSqlCheckIssue, PlpgSqlCheckResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ByteOffset(usize);

impl ByteOffset {
    fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OneBasedCharOffset(usize);

/// Find the first occurrence of target text that is not within string literals.
///
/// Returned offsets are byte offsets into `text`, because `TextRange` uses UTF-8
/// byte offsets internally. Iterate with `char_indices` so every slice starts on
/// a valid UTF-8 character boundary.
fn find_text_outside_strings(text: &str, target: &str) -> Option<ByteOffset> {
    let mut in_string = false;
    let mut quote_char = '\0';
    let bytes = text.as_bytes();
    let mut chars = text.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if !in_string {
            // Check if we're starting a string literal
            if ch == '\'' || ch == '"' {
                in_string = true;
                quote_char = ch;
            } else if let Some(candidate) = text[i..].get(..target.len()) {
                // Check if we found our target at this position. The query text
                // emitted by plpgsql_check is SQL, so ASCII-insensitive matching
                // is sufficient and avoids lowercasing into a string with
                // potentially different byte offsets.
                if candidate == target || candidate.eq_ignore_ascii_case(target) {
                    // Check if this is a complete word (not part of another identifier)
                    let is_word_start =
                        i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
                    let target_end = i + target.len();
                    let is_word_end = target_end >= bytes.len()
                        || (!bytes[target_end].is_ascii_alphanumeric()
                            && bytes[target_end] != b'_');

                    if is_word_start && is_word_end {
                        return Some(ByteOffset(i));
                    }
                }
            }
        } else {
            // We're inside a string literal
            if ch == quote_char {
                // Check if it's escaped (look for double quotes/apostrophes)
                if let Some((_, next_ch)) = chars.peek()
                    && *next_ch == quote_char
                {
                    // Skip the escaped quote
                    chars.next();
                } else {
                    // End of string literal
                    in_string = false;
                    quote_char = '\0';
                }
            }
        }
    }

    None
}

fn byte_offset_from_one_based_char_offset(
    text: &str,
    position: OneBasedCharOffset,
) -> Option<ByteOffset> {
    let zero_based_position = position.0.checked_sub(1)?;

    match text.char_indices().nth(zero_based_position) {
        Some((offset, _)) => Some(ByteOffset(offset)),
        None if zero_based_position == text.chars().count() => Some(ByteOffset(text.len())),
        None => None,
    }
}

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
    /// the relation (table or view) where the issue was found, if applicable
    /// only applicable for trigger functions
    pub relation: Option<String>,
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

        // Show relation information if available
        if let Some(relation) = &self.relation {
            visitor.record_log(
                LogCategory::Info,
                &markup! { "Relation: " <Emphasis>{relation}</Emphasis> },
            )?;
        }

        Ok(())
    }
}

/// Convert plpgsql_check results into diagnostics with optional relation info for triggers
pub fn create_diagnostics_from_check_result(
    result: &PlpgSqlCheckResult,
    fn_body: &str,
    offset: usize,
    relation: Option<String>,
) -> Vec<PlPgSqlCheckDiagnostic> {
    result
        .issues
        .iter()
        .map(|issue| {
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
                    relation: relation.clone(),
                },
            }
        })
        .collect()
}

fn resolve_span(issue: &PlpgSqlCheckIssue, fn_body: &str, offset: usize) -> Option<TextRange> {
    let stmt = match issue.statement.as_ref() {
        Some(s) => s,
        None => {
            let leading_whitespace = fn_body.len() - fn_body.trim_ascii_start().len();
            let trailing_whitespace = fn_body.len() - fn_body.trim_ascii_end().len();

            return Some(TextRange::new(
                (offset + leading_whitespace).try_into().unwrap(),
                (offset + fn_body.len() - trailing_whitespace)
                    .try_into()
                    .unwrap(),
            ));
        }
    };

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
        // first find the query within the fn body *after* stmt_offset, ignoring string literals
        let query_start = find_text_outside_strings(&fn_body[stmt_offset..], &q.text)
            .map(|pos| ByteOffset(pos.get() + stmt_offset));

        // The position is *within* the query text. plpgsql_check/Postgres
        // reports it as a 1-based character position, while TextRange uses
        // UTF-8 byte offsets.
        let pos = byte_offset_from_one_based_char_offset(
            &q.text,
            OneBasedCharOffset(
                q.position
                    .parse::<usize>()
                    .expect("Expected query position to be a valid usize"),
            ),
        )?;

        let start = query_start?.get() + pos.get();

        // the range of the diagnostics is the token that `pos` is on
        // Find the end of the current token by looking for whitespace or SQL delimiters
        let remaining = &fn_body[start..];
        let end = remaining
            .char_indices()
            .find(|(_, c)| {
                c.is_whitespace() || matches!(c, ',' | ';' | ')' | '(' | '=' | '<' | '>')
            })
            .map(|(i, _c)| {
                i // just the token end, don't include delimiters
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Query, Statement};

    fn diagnostic_span_for(fn_body: &str, query_text: &str, position: usize) -> String {
        let result = PlpgSqlCheckResult {
            function: "public.f1".to_string(),
            issues: vec![PlpgSqlCheckIssue {
                level: "error".to_string(),
                message: "column does not exist".to_string(),
                statement: Some(Statement {
                    line_number: "2".to_string(),
                    text: "RETURN QUERY".to_string(),
                }),
                query: Some(Query {
                    position: position.to_string(),
                    text: query_text.to_string(),
                }),
                sql_state: Some("42703".to_string()),
            }],
        };

        let diagnostics = create_diagnostics_from_check_result(&result, fn_body, 0, None);
        let span = diagnostics[0].span.expect("expected diagnostic span");
        fn_body[usize::from(span.start())..usize::from(span.end())].to_string()
    }

    #[test]
    fn maps_query_span_when_multibyte_comment_precedes_query() {
        let query = "SELECT missing_column FROM t";
        let fn_body = format!("BEGIN\n  RETURN QUERY\n    -- prüfen\n    {query};\nEND");

        assert_eq!(diagnostic_span_for(&fn_body, query, 8), "missing_column");
    }

    #[test]
    fn maps_query_position_as_character_offset() {
        let query = "SELECT 'prüfen', missing_column FROM t";
        let fn_body = format!("BEGIN\n  RETURN QUERY {query};\nEND");
        let position = query
            .chars()
            .position(|ch| ch == 'm')
            .expect("query should contain target column")
            + 1;

        assert_eq!(
            diagnostic_span_for(&fn_body, query, position),
            "missing_column"
        );
    }
}
