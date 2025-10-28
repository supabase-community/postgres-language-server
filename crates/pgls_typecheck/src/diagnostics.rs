use std::io;

use once_cell::sync::Lazy;
use pgls_console::markup;
use pgls_diagnostics::{Advices, Diagnostic, LogCategory, MessageAndDescription, Severity, Visit};
use pgls_text_size::{TextRange, TextSize};
use regex::Regex;
use sqlx::postgres::{PgDatabaseError, PgSeverity};

use crate::typed_identifier::{IdentifierReplacement, TypedReplacement};

/// A specialized diagnostic for the typechecker.
///
/// Type diagnostics are always **errors**.
#[derive(Clone, Debug, Diagnostic)]
#[diagnostic(category = "typecheck")]
pub struct TypecheckDiagnostic {
    #[location(span)]
    span: Option<TextRange>,
    #[description]
    #[message]
    message: MessageAndDescription,
    #[advice]
    advices: TypecheckAdvices,
    #[severity]
    severity: Severity,
}

#[derive(Debug, Clone)]
struct TypecheckAdvices {
    code: String,
    schema: Option<String>,
    table: Option<String>,
    column: Option<String>,
    data_type: Option<String>,
    constraint: Option<String>,
    detail: Option<String>,
    where_: Option<String>,
    hint: Option<String>,

    #[allow(unused)]
    line: Option<usize>,
    #[allow(unused)]
    file: Option<String>,
    #[allow(unused)]
    routine: Option<String>,
}

impl Advices for TypecheckAdvices {
    fn record(&self, visitor: &mut dyn Visit) -> io::Result<()> {
        // First, show the error code
        visitor.record_log(
            LogCategory::Error,
            &markup! { "Error Code: " <Emphasis>{&self.code}</Emphasis> },
        )?;

        // Show detailed message if available
        if let Some(detail) = &self.detail {
            visitor.record_log(LogCategory::Info, &detail)?;
        }

        // Show object location information
        if let (Some(schema), Some(table)) = (&self.schema, &self.table) {
            let mut location = format!("In table: {schema}.{table}");
            if let Some(column) = &self.column {
                location.push_str(&format!(", column: {column}"));
            }
            visitor.record_log(LogCategory::Info, &location)?;
        }

        // Show constraint information
        if let Some(constraint) = &self.constraint {
            visitor.record_log(
                LogCategory::Info,
                &markup! { "Constraint: " <Emphasis>{constraint}</Emphasis> },
            )?;
        }

        // Show data type information
        if let Some(data_type) = &self.data_type {
            visitor.record_log(
                LogCategory::Info,
                &markup! { "Data type: " <Emphasis>{data_type}</Emphasis> },
            )?;
        }

        // Show context information
        if let Some(where_) = &self.where_ {
            visitor.record_log(LogCategory::Info, &markup! { "Context:\n"{where_}"" })?;
        }

        // Show hint if available
        if let Some(hint) = &self.hint {
            visitor.record_log(LogCategory::Info, &markup! { "Hint: "{hint}"" })?;
        }

        Ok(())
    }
}

/// Pattern and rewrite rule for error messages
struct ErrorRewriteRule {
    pattern: Regex,
    rewrite: fn(&regex::Captures, &IdentifierReplacement) -> String,
}

static ERROR_REWRITE_RULES: Lazy<Vec<ErrorRewriteRule>> = Lazy::new(|| {
    vec![
        ErrorRewriteRule {
            pattern: Regex::new(r#"invalid input syntax for type ([\w\s]+): "([^"]*)""#).unwrap(),
            rewrite: |caps, replacement| {
                let expected_type = &caps[1];
                format!(
                    "`{}` is of type {}, not {}",
                    replacement.original_name, replacement.type_name, expected_type
                )
            },
        },
        ErrorRewriteRule {
            pattern: Regex::new(r#"operator does not exist: (.+)"#).unwrap(),
            rewrite: |caps, replacement| {
                let operator_expr = &caps[1];
                format!(
                    "operator does not exist: {} (parameter `{}` is of type {})",
                    operator_expr, replacement.original_name, replacement.type_name
                )
            },
        },
    ]
});

/// Rewrites Postgres error messages to be more user-friendly
pub fn rewrite_error_message(
    pg_error_message: &str,
    replacement: &IdentifierReplacement,
) -> String {
    for rule in ERROR_REWRITE_RULES.iter() {
        if let Some(caps) = rule.pattern.captures(pg_error_message) {
            return (rule.rewrite)(&caps, replacement);
        }
    }

    // if we don't have a matching error-rewrite-rule,
    // we'll fallback to replacing default values with their types,
    // e.g. `""` is replaced with `text`.
    let unquoted_default = replacement.default_value.trim_matches('\'');
    pg_error_message
        .replace(&format!("\"{unquoted_default}\""), &replacement.type_name)
        .replace(&format!("'{unquoted_default}'"), &replacement.type_name)
}

pub(crate) fn create_type_error(
    pg_err: &PgDatabaseError,
    ts: &tree_sitter::Tree,
    typed_replacement: TypedReplacement,
) -> TypecheckDiagnostic {
    let position = pg_err.position().and_then(|pos| match pos {
        sqlx::postgres::PgErrorPosition::Original(pos) => Some(pos - 1),
        _ => None,
    });

    let original_position = position.map(|p| {
        let pos = TextSize::new(p.try_into().unwrap());

        typed_replacement
            .text_replacement()
            .to_original_position(pos)
    });

    let range = original_position.and_then(|pos| {
        ts.root_node()
            .named_descendant_for_byte_range(pos.into(), pos.into())
            .map(|node| {
                TextRange::new(
                    node.start_byte().try_into().unwrap(),
                    node.end_byte().try_into().unwrap(),
                )
            })
    });

    let severity = match pg_err.severity() {
        PgSeverity::Panic => Severity::Error,
        PgSeverity::Fatal => Severity::Error,
        PgSeverity::Error => Severity::Error,
        PgSeverity::Warning => Severity::Warning,
        PgSeverity::Notice => Severity::Hint,
        PgSeverity::Debug => Severity::Hint,
        PgSeverity::Info => Severity::Information,
        PgSeverity::Log => Severity::Information,
    };

    let message = if let Some(pos) = original_position {
        if let Some(replacement) = typed_replacement.find_type_at_position(pos) {
            rewrite_error_message(pg_err.message(), replacement)
        } else {
            pg_err.to_string()
        }
    } else {
        pg_err.to_string()
    };

    TypecheckDiagnostic {
        message: message.into(),
        severity,
        span: range,
        advices: TypecheckAdvices {
            code: pg_err.code().to_string(),
            hint: pg_err.hint().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            schema: pg_err.schema().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            table: pg_err.table().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            detail: pg_err.detail().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            column: pg_err.column().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            data_type: pg_err.data_type().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            constraint: pg_err.constraint().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            line: pg_err.line(),
            file: pg_err.file().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            routine: pg_err.routine().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
            where_: pg_err.r#where().and_then(|s| {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }),
        },
    }
}
