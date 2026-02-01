use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{
    JsonEncoding, JsonFormat, JsonFormatType, JsonOutput, JsonReturning, JsonValueExpr,
};

pub(super) fn emit_json_value_expr(e: &mut EventEmitter, n: &JsonValueExpr) {
    e.group_start(GroupKind::JsonValueExpr);

    if let Some(ref raw_expr) = n.raw_expr {
        super::emit_node(raw_expr, e);
    } else if let Some(ref formatted_expr) = n.formatted_expr {
        super::emit_node(formatted_expr, e);
    }

    if let Some(ref format) = n.format {
        emit_json_format(e, format);
    }

    e.group_end();
}

pub(super) fn emit_json_output_node(e: &mut EventEmitter, n: &JsonOutput) {
    e.group_start(GroupKind::JsonOutput);

    e.token(TokenKind::RETURNING_KW);

    if let Some(ref type_name) = n.type_name {
        e.space();
        super::emit_type_name(e, type_name);
    }

    if let Some(ref returning) = n.returning {
        let mut has_content = true;
        emit_json_returning_clause(e, returning, &mut has_content);
    }

    e.group_end();
}

pub(super) fn emit_json_output(e: &mut EventEmitter, output: &JsonOutput, has_content: &mut bool) {
    if *has_content {
        e.space();
    }

    e.token(TokenKind::RETURNING_KW);

    if let Some(ref type_name) = output.type_name {
        e.space();
        super::emit_type_name(e, type_name);
    }

    if let Some(ref returning) = output.returning {
        emit_json_returning_clause(e, returning, has_content);
    }

    *has_content = true;
}

pub(super) fn emit_json_format_node(e: &mut EventEmitter, format: &JsonFormat) {
    e.group_start(GroupKind::JsonFormat);

    emit_json_format_without_prefix(e, format);

    e.group_end();
}

pub(super) fn emit_json_format(e: &mut EventEmitter, format: &JsonFormat) {
    emit_json_format_with_prefix(e, format, true);
}

fn emit_json_format_without_prefix(e: &mut EventEmitter, format: &JsonFormat) {
    emit_json_format_with_prefix(e, format, false);
}

fn emit_json_format_with_prefix(e: &mut EventEmitter, format: &JsonFormat, prefix_space: bool) {
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
            e.token(TokenKind::IDENT("JSONB".to_string()));
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

pub(super) fn emit_json_returning_node(e: &mut EventEmitter, returning: &JsonReturning) {
    e.group_start(GroupKind::JsonReturning);

    let mut has_content = false;
    emit_json_returning_clause(e, returning, &mut has_content);

    e.group_end();
}

pub(super) fn emit_json_returning_clause(
    e: &mut EventEmitter,
    returning: &JsonReturning,
    _has_content: &mut bool,
) {
    // JsonReturning contains internal type info (typid, typmod), not SQL RETURNING clause
    // Only emit the format if present (FORMAT JSON ENCODING ...)
    if let Some(ref format) = returning.format {
        emit_json_format(e, format);
    }
}

fn emit_encoding(e: &mut EventEmitter, label: &str, needs_space: bool) {
    if needs_space {
        e.space();
    }
    e.token(TokenKind::ENCODING_KW);
    e.space();
    e.token(TokenKind::IDENT(label.to_string()));
}
