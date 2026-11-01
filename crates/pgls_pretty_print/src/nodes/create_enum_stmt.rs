use pgls_query::protobuf::CreateEnumStmt;

use super::prelude::*;

pub(super) fn emit_create_enum_stmt(e: &mut EventEmitter, n: &CreateEnumStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);

    // Emit the type name (qualified name as a list)
    if !n.type_name.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.type_name);
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    e.token(TokenKind::ENUM_KW);

    // Emit the enum values list (as string literals with quotes)
    if !n.vals.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.vals, |e, n| {
            let s = assert_node_variant!(String, n);
            emit_string_literal(e, s);
        });
    }

    e.group_end();
}
