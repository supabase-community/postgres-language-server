use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::JsonSerializeExpr;

use super::json_value_expr::{emit_json_output, emit_json_value_expr};

pub(super) fn emit_json_serialize_expr(e: &mut EventEmitter, n: &JsonSerializeExpr) {
    e.group_start(GroupKind::JsonSerializeExpr);

    e.token(TokenKind::IDENT("JSON_SERIALIZE".to_string()));
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref expr) = n.expr {
        emit_json_value_expr(e, expr);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
