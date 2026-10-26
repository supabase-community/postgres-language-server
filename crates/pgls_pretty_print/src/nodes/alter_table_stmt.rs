use pgls_query::protobuf::{AlterTableStmt, ObjectType};

use super::prelude::*;

pub(super) fn emit_alter_table_stmt(e: &mut EventEmitter, n: &AlterTableStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type (TABLE, INDEX, TYPE, etc.)
    emit_keywords(e, object_type_keywords(n.objtype()).unwrap_or("TABLE"));

    e.space();

    if n.missing_ok {
        emit_if_exists(e);
        e.space();
    }

    // Emit relation name
    // For ObjectType (ALTER TYPE), don't emit ONLY since it's not valid
    if let Some(ref relation) = n.relation {
        match n.objtype() {
            ObjectType::ObjectType => super::emit_range_var_as(e, relation, super::Only::Omitted),
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
            super::emit_alter_table_cmd_of(e, cmd, n.objtype());
        }

        e.indent_end();
    }

    e.group_end();
}
