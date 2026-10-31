//! The clauses of SQL/JSON: `RETURNING type FORMAT JSON ENCODING UTF8`, and the behaviors
//! `... ON EMPTY` and `... ON ERROR`.

use pgls_query::protobuf::{
    JsonBehavior, JsonBehaviorType, JsonEncoding, JsonFormat, JsonFormatType, JsonOutput,
    JsonReturning,
};

use crate::nodes::prelude::*;

pub(crate) fn emit_json_output_clause(
    e: &mut EventEmitter,
    output: &JsonOutput,
    has_content: &mut bool,
) {
    if *has_content {
        e.space();
    }

    e.token(TokenKind::RETURNING_KW);

    if let Some(ref type_name) = output.type_name {
        e.space();
        crate::nodes::emit_type_name(e, type_name);
    }

    if let Some(ref returning) = output.returning {
        emit_json_returning_clause(e, returning, has_content);
    }

    *has_content = true;
}

pub(crate) fn emit_json_returning_clause(
    e: &mut EventEmitter,
    returning: &JsonReturning,
    _has_content: &mut bool,
) {
    // JsonReturning contains internal type info (typid, typmod), not SQL RETURNING clause
    // Only emit the format if present (FORMAT JSON ENCODING ...)
    if let Some(ref format) = returning.format {
        emit_json_format_clause(e, format);
    }
}

pub(crate) fn emit_json_format_clause(e: &mut EventEmitter, format: &JsonFormat) {
    emit_json_format_after(e, format, Prefix::Space);
}

/// What comes before a FORMAT clause.
#[derive(Clone, Copy)]
pub(crate) enum Prefix {
    /// A space: the clause follows an expression on the same line.
    Space,
    /// Nothing: the clause is the whole node.
    None,
}

pub(crate) fn emit_json_format_after(e: &mut EventEmitter, format: &JsonFormat, prefix: Prefix) {
    let prefix_space = matches!(prefix, Prefix::Space);
    let mut wrote_any = false;

    match format.format_type() {
        JsonFormatType::JsFormatJson => {
            if prefix_space {
                e.space();
            }
            e.token(TokenKind::FORMAT_KW);
            e.space();
            e.token(TokenKind::JSON_KW);
            wrote_any = true;
        }
        JsonFormatType::JsFormatJsonb => {
            if prefix_space {
                e.space();
            }
            e.token(TokenKind::FORMAT_KW);
            e.space();
            emit_keyword(e, "jsonb");
            wrote_any = true;
        }
        JsonFormatType::Undefined | JsonFormatType::JsFormatDefault => {}
    }

    match format.encoding() {
        JsonEncoding::JsEncUtf8 => emit_encoding(e, "UTF8", prefix_space || wrote_any),
        JsonEncoding::JsEncUtf16 => emit_encoding(e, "UTF16", prefix_space || wrote_any),
        JsonEncoding::JsEncUtf32 => emit_encoding(e, "UTF32", prefix_space || wrote_any),
        JsonEncoding::Undefined | JsonEncoding::JsEncDefault => {}
    }
}

fn emit_encoding(e: &mut EventEmitter, label: &str, needs_space: bool) {
    if needs_space {
        e.space();
    }
    e.token(TokenKind::ENCODING_KW);
    e.space();
    emit_keyword(e, label);
}

/// Emits a behavior of SQL/JSON, `NULL`, `EMPTY ARRAY` or `DEFAULT expr`, without the `ON EMPTY` or
/// `ON ERROR` that follows it.
pub(crate) fn emit_json_behavior_keywords(e: &mut EventEmitter, behavior: &JsonBehavior) {
    match JsonBehaviorType::try_from(behavior.btype).unwrap_or(JsonBehaviorType::Undefined) {
        JsonBehaviorType::JsonBehaviorNull => e.token(TokenKind::NULL_KW),
        JsonBehaviorType::JsonBehaviorError => e.token(TokenKind::ERROR_KW),
        JsonBehaviorType::JsonBehaviorEmpty => e.token(TokenKind::EMPTY_KW),
        JsonBehaviorType::JsonBehaviorTrue => e.token(TokenKind::TRUE_KW),
        JsonBehaviorType::JsonBehaviorFalse => e.token(TokenKind::FALSE_KW),
        JsonBehaviorType::JsonBehaviorUnknown => e.token(TokenKind::UNKNOWN_KW),
        JsonBehaviorType::JsonBehaviorEmptyArray => {
            e.token(TokenKind::EMPTY_KW);
            e.space();
            e.token(TokenKind::ARRAY_KW);
        }
        JsonBehaviorType::JsonBehaviorEmptyObject => {
            e.token(TokenKind::EMPTY_KW);
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        JsonBehaviorType::JsonBehaviorDefault => {
            e.token(TokenKind::DEFAULT_KW);
            if let Some(expr) = behavior.expr.as_ref() {
                e.space();
                emit_node(e, expr);
            } else {
                debug_assert!(false, "DEFAULT json behavior requires an expression");
            }
        }
        JsonBehaviorType::Undefined => {
            debug_assert!(false, "Undefined JSON behavior encountered");
        }
    }
}

pub(crate) enum JsonBehaviorClause {
    OnEmpty,
    OnError,
}

pub(crate) fn emit_json_behavior_clause(
    e: &mut EventEmitter,
    behavior: &JsonBehavior,
    clause: JsonBehaviorClause,
) {
    emit_json_behavior_keywords(e, behavior);
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();
    match clause {
        JsonBehaviorClause::OnEmpty => e.token(TokenKind::EMPTY_KW),
        JsonBehaviorClause::OnError => e.token(TokenKind::ERROR_KW),
    }
}
