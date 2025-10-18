use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::TypeCast;

pub(super) fn emit_type_cast(e: &mut EventEmitter, n: &TypeCast) {
    e.group_start(GroupKind::TypeCast);

    // CAST(expr AS type) syntax
    e.token(TokenKind::CAST_KW);
    e.token(TokenKind::L_PAREN);

    // Emit the expression
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();

    // Emit the type
    if let Some(ref type_name) = n.type_name {
        super::emit_type_name(e, type_name);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
