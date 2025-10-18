use pgt_query::protobuf::CreateEnumStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_create_enum_stmt(e: &mut EventEmitter, n: &CreateEnumStmt) {
    e.group_start(GroupKind::CreateEnumStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);

    // Emit the type name (qualified name as a list)
    if !n.type_name.is_empty() {
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.type_name);
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    e.token(TokenKind::ENUM_KW);

    // Emit the enum values list (as string literals with quotes)
    if !n.vals.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.vals, |n, e| {
            let s = assert_node_variant!(String, n);
            super::string::emit_string_literal(e, s);
        });
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
