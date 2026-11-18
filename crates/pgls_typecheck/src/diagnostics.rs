use std::io;

use pgls_console::markup;
use pgls_diagnostics::{Advices, Diagnostic, LogCategory, MessageAndDescription, Severity, Visit};
use pgls_line_index::offset_of_char_position;
use pgls_text_size::{TextRange, TextSize};
use sqlx::postgres::{PgDatabaseError, PgSeverity};

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

pub(crate) fn create_type_error(
    pg_err: &PgDatabaseError,
    ts: &tree_sitter::Tree,
    sql: &str,
    source_offset: TextSize,
) -> TypecheckDiagnostic {
    let position = pg_err.position().and_then(|pos| match pos {
        sqlx::postgres::PgErrorPosition::Original(pos) => offset_of_char_position(sql, pos),
        _ => None,
    });

    let range = position.and_then(|pos| {
        ts.root_node()
            .named_descendant_for_byte_range(pos.into(), pos.into())
            .and_then(|node| {
                let node = if source_offset == TextSize::from(0) {
                    node
                } else {
                    node.parent()
                        .filter(|parent| parent.kind() == "object_reference")
                        .and_then(|parent| {
                            let mut cursor = parent.walk();
                            parent
                                .children(&mut cursor)
                                .filter(|child| child.is_named())
                                .last()
                        })
                        .unwrap_or(node)
                };

                Some(TextRange::new(
                    TextSize::from(u32::try_from(node.start_byte()).ok()?) + source_offset,
                    TextSize::from(u32::try_from(node.end_byte()).ok()?) + source_offset,
                ))
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

    TypecheckDiagnostic {
        message: pg_err.to_string().into(),
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
