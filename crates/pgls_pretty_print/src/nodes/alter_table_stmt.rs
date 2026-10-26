use pgls_query::protobuf::{AlterTableStmt, ObjectType};

use super::prelude::*;

pub(super) fn emit_alter_table_stmt(e: &mut EventEmitter, n: &AlterTableStmt) {
    e.group_start(GroupKind::AlterTableStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type (TABLE, INDEX, TYPE, etc.)
    match n.objtype() {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
        ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
        ObjectType::ObjectMatview => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
        ObjectType::ObjectForeignTable => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::TABLE_KW);
        }
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        _ => e.token(TokenKind::TABLE_KW), // Default to TABLE
    }

    e.space();

    if n.missing_ok {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    // Emit relation name
    // For ObjectType (ALTER TYPE), don't emit ONLY since it's not valid
    if let Some(ref relation) = n.relation {
        match n.objtype() {
            ObjectType::ObjectType => super::emit_range_var_name(e, relation),
            _ => super::emit_range_var(e, relation),
        }
    }

    // Emit commands - each on its own indented line
    if !n.cmds.is_empty() {
        e.indent_start();
        e.line(LineType::Hard);

        for (i, cmd_node) in n.cmds.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::Hard);
            }

            // Extract AlterTableCmd from Node
            let cmd = assert_node_variant!(AlterTableCmd, cmd_node);
            let for_type = matches!(n.objtype(), ObjectType::ObjectType);
            super::emit_alter_table_cmd_for_type(e, cmd, for_type);
        }

        e.indent_end();
    }

    e.group_end();
}
