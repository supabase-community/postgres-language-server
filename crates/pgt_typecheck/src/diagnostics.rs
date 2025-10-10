use std::io;

use pgt_console::markup;
use pgt_diagnostics::{Advices, Diagnostic, LogCategory, MessageAndDescription, Severity, Visit};
use pgt_text_size::{TextRange, TextSize};
use sqlx::postgres::{PgDatabaseError, PgSeverity};

use crate::{IdentifierType, TypedReplacement};

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

/// Finds the original type at the given position in the adjusted text
fn find_type_at_position(
    adjusted_position: TextSize,
    type_info: &[(TextRange, IdentifierType)],
) -> Option<&IdentifierType> {
    type_info
        .iter()
        .find(|(range, _)| range.contains(adjusted_position))
        .map(|(_, type_)| type_)
}

/// Rewrites error messages to show the original type name instead of the replaced literal value
fn rewrite_error_message(original_message: &str, identifier_type: &IdentifierType) -> String {
    // pattern: invalid input syntax for type X: "literal_value"
    // we want to replace "literal_value" with the type name

    if let Some(colon_pos) = original_message.rfind(": ") {
        let before_value = &original_message[..colon_pos];

        // build the type name, including schema if present
        let type_name = if let Some(schema) = &identifier_type.schema {
            format!("{}.{}", schema, identifier_type.name)
        } else {
            identifier_type.name.clone()
        };

        format!("{}: {}", before_value, type_name)
    } else {
        original_message.to_string()
    }
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

    let range = position.and_then(|pos| {
        let adjusted = typed_replacement.replacement.to_original_position(TextSize::new(pos.try_into().unwrap()));

        ts.root_node()
            .named_descendant_for_byte_range(adjusted.into(), adjusted.into())
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

    // check if the error position corresponds to a replaced parameter
    let message = if let Some(pos) = position {
        let adjusted_pos = TextSize::new(pos.try_into().unwrap());
        if let Some(original_type) = find_type_at_position(adjusted_pos, &typed_replacement.type_info) {
            rewrite_error_message(pg_err.message(), original_type)
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
