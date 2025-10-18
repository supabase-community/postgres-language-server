use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::JsonScalarExpr;

pub(super) fn emit_json_scalar_expr(e: &mut EventEmitter, n: &JsonScalarExpr) {
    e.group_start(GroupKind::JsonScalarExpr);

    e.token(TokenKind::IDENT("JSON_SCALAR".to_string()));
    e.token(TokenKind::L_PAREN);

    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
