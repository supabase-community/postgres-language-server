use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::JsonParseExpr;

pub(super) fn emit_json_parse_expr(e: &mut EventEmitter, n: &JsonParseExpr) {
    e.group_start(GroupKind::JsonParseExpr);

    e.token(TokenKind::IDENT("JSON".to_string()));
    e.token(TokenKind::L_PAREN);

    if let Some(ref expr) = n.expr {
        if let Some(ref raw_expr) = expr.raw_expr {
            super::emit_node(raw_expr, e);
        }
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
