use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::{AlterTableMoveAllStmt, ObjectType};

pub(super) fn emit_alter_table_move_all_stmt(e: &mut EventEmitter, n: &AlterTableMoveAllStmt) {
    e.group_start(GroupKind::AlterTableMoveAllStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type (TABLE, INDEX, MATERIALIZED VIEW)
    match n.objtype() {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
        ObjectType::ObjectMatview => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        _ => e.token(TokenKind::TABLE_KW), // Default to TABLE
    }

    e.space();
    e.token(TokenKind::ALL_KW);
    e.space();
    e.token(TokenKind::IN_KW);
    e.space();
    e.token(TokenKind::IDENT("TABLESPACE".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.orig_tablespacename.clone()));

    // Emit OWNED BY roles if specified
    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("OWNED".to_string()));
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::node_list::emit_comma_separated_list(e, &n.roles, |node, e| {
            let role_spec = assert_node_variant!(RoleSpec, node);
            super::emit_role_spec(e, role_spec);
        });
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::SET_KW);
    e.space();
    e.token(TokenKind::IDENT("TABLESPACE".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.new_tablespacename.clone()));

    if n.nowait {
        e.space();
        e.token(TokenKind::IDENT("NOWAIT".to_string()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
