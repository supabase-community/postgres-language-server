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
    pub statement: Option<String>,
    pub query: Option<String>,
    pub line_number: Option<String>,
    pub query_position: Option<String>,
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

        // Show statement information if available
        if let Some(statement) = &self.statement {
            if let Some(line_number) = &self.line_number {
                visitor.record_log(
                    LogCategory::Info,
                    &markup! { "At line " <Emphasis>{line_number}</Emphasis> ": "{statement}"" },
                )?;
            } else {
                visitor.record_log(LogCategory::Info, &markup! { "Statement: "{statement}"" })?;
            }
        }

        // Show query information if available
        if let Some(query) = &self.query {
            if let Some(pos) = &self.query_position {
                visitor.record_log(
                    LogCategory::Info,
                    &markup! { "In query at position " <Emphasis>{pos}</Emphasis> ":\n"{query}"" },
                )?;
            } else {
                visitor.record_log(LogCategory::Info, &markup! { "Query:\n"{query}"" })?;
            }
        }

        Ok(())
    }
}

/// Convert plpgsql_check results into diagnostics
pub fn create_diagnostics_from_check_result(
    result: &PlpgSqlCheckResult,
    fn_body: &str,
    start: usize,
) -> Vec<PlPgSqlCheckDiagnostic> {
    result
        .issues
        .iter()
        .map(|issue| create_diagnostic_from_issue(issue, fn_body, start))
        .collect()
}

fn create_diagnostic_from_issue(
    issue: &PlpgSqlCheckIssue,
    fn_body: &str,
    start: usize,
) -> PlPgSqlCheckDiagnostic {
    let severity = match issue.level.as_str() {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        "notice" => Severity::Hint,
        _ => Severity::Information,
    };

    let span = if let Some(s) = &issue.statement {
        let line_number = s.line_number.parse::<usize>().unwrap_or(0);
        if line_number > 0 {
            let mut current_offset = 0;
            let mut result = None;
            for (i, line) in fn_body.lines().enumerate() {
                if i + 1 == line_number {
                    if let Some(stmt_pos) = line.to_lowercase().find(&s.text.to_lowercase()) {
                        let line_start = start + current_offset + stmt_pos;
                        let line_end = line_start + s.text.len();
                        result = Some(TextRange::new(
                            (line_start as u32).into(),
                            (line_end as u32).into(),
                        ));
                    } else {
                        let line_start = start + current_offset;
                        let line_end = line_start + line.len();
                        result = Some(TextRange::new(
                            (line_start as u32).into(),
                            (line_end as u32).into(),
                        ));
                    }
                    break;
                }
                current_offset += line.len() + 1;
            }
            result
        } else {
            None
        }
    } else {
        None
    };

    PlPgSqlCheckDiagnostic {
        message: issue.message.clone().into(),
        severity,
        span,
        advices: PlPgSqlCheckAdvices {
            code: issue.sql_state.clone(),
            statement: issue.statement.as_ref().map(|s| s.text.clone()),
            query: issue.query.as_ref().map(|q| q.text.clone()),
            line_number: issue.statement.as_ref().map(|s| s.line_number.clone()),
            query_position: issue.query.as_ref().map(|q| q.position.clone()),
        },
    }
}
