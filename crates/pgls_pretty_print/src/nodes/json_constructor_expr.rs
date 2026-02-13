use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::{JsonConstructorExpr, JsonConstructorType};
use std::convert::TryFrom;

pub(super) fn emit_json_constructor_expr(e: &mut EventEmitter, n: &JsonConstructorExpr) {
    e.group_start(GroupKind::JsonConstructorExpr);

    let constructor =
        JsonConstructorType::try_from(n.r#type).unwrap_or(JsonConstructorType::Undefined);
    e.token(TokenKind::IDENT(
        constructor_keyword(constructor).to_string(),
    ));
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if !n.args.is_empty() {
        emit_comma_separated_list(e, &n.args, super::emit_node);
        has_content = true;
    }

    if let Some(func) = n.func.as_ref() {
        if has_content {
            e.space();
        }
        super::emit_node(func, e);
        has_content = true;
    }

    if let Some(coercion) = n.coercion.as_ref() {
        if has_content {
            e.space();
        }
        super::emit_node(coercion, e);
        has_content = true;
    }

    if let Some(returning) = n.returning.as_ref() {
        super::json_value_expr::emit_json_returning_clause(e, returning, &mut has_content);
    }

    if n.absent_on_null {
        if has_content {
            e.space();
        }
        e.token(TokenKind::ABSENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
        has_content = true;
    }

    if (matches!(constructor, JsonConstructorType::JsctorJsonObject)
        || matches!(constructor, JsonConstructorType::JsctorJsonObjectagg))
        && n.unique
    {
        if has_content {
            e.space();
        }
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::UNIQUE_KW);
        e.space();
        e.token(TokenKind::KEYS_KW);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}

fn constructor_keyword(kind: JsonConstructorType) -> &'static str {
    match kind {
        JsonConstructorType::JsctorJsonObject => "JSON_OBJECT",
        JsonConstructorType::JsctorJsonArray => "JSON_ARRAY",
        JsonConstructorType::JsctorJsonObjectagg => "JSON_OBJECTAGG",
        JsonConstructorType::JsctorJsonArrayagg => "JSON_ARRAYAGG",
        JsonConstructorType::JsctorJsonParse => "JSON_PARSE",
        JsonConstructorType::JsctorJsonScalar => "JSON_SCALAR",
        JsonConstructorType::JsctorJsonSerialize => "JSON_SERIALIZE",
        JsonConstructorType::Undefined => "JSON_CONSTRUCTOR",
    }
}
