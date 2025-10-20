use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{
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
        emit_json_returning(e, returning);
    }

    *has_content = true;
}

pub(super) fn emit_json_format(e: &mut EventEmitter, format: &JsonFormat) {
    match format.format_type() {
        JsonFormatType::JsFormatJson => {
            e.space();
            e.token(TokenKind::FORMAT_KW);
            e.space();
            e.token(TokenKind::JSON_KW);
        }
        JsonFormatType::JsFormatJsonb => {
            e.space();
            e.token(TokenKind::FORMAT_KW);
            e.space();
            e.token(TokenKind::IDENT("JSONB".to_string()));
        }
        JsonFormatType::Undefined | JsonFormatType::JsFormatDefault => {}
    }

    match format.encoding() {
        JsonEncoding::JsEncUtf8 => emit_encoding(e, "UTF8"),
        JsonEncoding::JsEncUtf16 => emit_encoding(e, "UTF16"),
        JsonEncoding::JsEncUtf32 => emit_encoding(e, "UTF32"),
        JsonEncoding::Undefined | JsonEncoding::JsEncDefault => {}
    }
}

fn emit_json_returning(e: &mut EventEmitter, returning: &JsonReturning) {
    if let Some(ref format) = returning.format {
        emit_json_format(e, format);
    }
}

fn emit_encoding(e: &mut EventEmitter, label: &str) {
    e.space();
    e.token(TokenKind::ENCODING_KW);
    e.space();
    e.token(TokenKind::IDENT(label.to_string()));
}
