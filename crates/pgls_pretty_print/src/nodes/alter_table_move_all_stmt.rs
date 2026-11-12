use pgls_query::protobuf::AlterTableMoveAllStmt;

use super::prelude::*;

pub(super) fn emit_alter_table_move_all_stmt(e: &mut EventEmitter, n: &AlterTableMoveAllStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type (TABLE, INDEX, MATERIALIZED VIEW)
    emit_keywords(e, object_type_keywords(n.objtype()).unwrap_or("TABLE"));

    e.space();
    e.token(TokenKind::ALL_KW);
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::IN_KW);
    e.space();
    e.token(TokenKind::TABLESPACE_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.orig_tablespacename);

    // Emit OWNED BY roles if specified
    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::OWNED_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.roles, |e, node| {
            let Some(NodeEnum::RoleSpec(role_spec)) = node.node.as_ref() else {
                emit_node(e, node);
                return;
            };
            super::emit_role_spec(e, role_spec);
        });
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::SET_KW);
    e.space();
    e.token(TokenKind::TABLESPACE_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.new_tablespacename);

    if n.nowait {
        e.space();
        e.token(TokenKind::NOWAIT_KW);
    }

    e.group_end();
}
