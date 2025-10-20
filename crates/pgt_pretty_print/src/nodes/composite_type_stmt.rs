use pgt_query::protobuf::CompositeTypeStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::emit_node;

pub(super) fn emit_composite_type_stmt(e: &mut EventEmitter, n: &CompositeTypeStmt) {
    e.group_start(GroupKind::CompositeTypeStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);
    e.space();

    if let Some(ref typevar) = n.typevar {
        super::emit_range_var(e, typevar);
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    e.token(TokenKind::L_PAREN);

    if !n.coldeflist.is_empty() {
        e.indent_start();
        e.line(LineType::SoftOrSpace);
        super::node_list::emit_comma_separated_list(e, &n.coldeflist, emit_node);
        e.indent_end();
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::R_PAREN);
    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
