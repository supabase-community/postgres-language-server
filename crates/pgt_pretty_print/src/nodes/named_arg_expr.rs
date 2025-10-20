use pgt_query::protobuf::NamedArgExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_named_arg_expr(e: &mut EventEmitter, n: &NamedArgExpr) {
    e.group_start(GroupKind::NamedArgExpr);

    // Emit the argument name
    if !n.name.is_empty() {
        super::emit_identifier(e, &n.name);
        e.space();
        e.token(TokenKind::IDENT(":=".to_string()));
        e.space();
    }

    // Emit the argument value
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }

    e.group_end();
}
