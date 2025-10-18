use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterOpFamilyStmt;

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_alter_op_family_stmt(e: &mut EventEmitter, n: &AlterOpFamilyStmt) {
    e.group_start(GroupKind::AlterOpFamilyStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("OPERATOR".to_string()));
    e.space();
    e.token(TokenKind::IDENT("FAMILY".to_string()));
    e.space();

    if !n.opfamilyname.is_empty() {
        emit_dot_separated_list(e, &n.opfamilyname);
    }

    e.space();
    e.token(TokenKind::IDENT("USING".to_string()));
    e.space();

    if !n.amname.is_empty() {
        e.token(TokenKind::IDENT(n.amname.clone()));
    }

    // Use indent and soft line break for ADD/DROP clause
    e.indent_start();
    e.line(crate::emitter::LineType::SoftOrSpace);

    // Start a group for the entire ADD/DROP clause to keep operator items together
    e.group_start(GroupKind::AlterOpFamilyStmt);

    if n.is_drop {
        e.token(TokenKind::DROP_KW);
    } else {
        e.token(TokenKind::ADD_KW);
    }

    if !n.items.is_empty() {
        e.space();
        // Emit items without comma separation to control line breaking
        // Each item should stay on its own line
        for (idx, item) in n.items.iter().enumerate() {
            if idx > 0 {
                e.token(TokenKind::IDENT(",".to_string()));
                e.line(crate::emitter::LineType::SoftOrSpace);
            }
            super::emit_node(item, e);
        }
    }

    e.group_end();
    e.indent_end();

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
