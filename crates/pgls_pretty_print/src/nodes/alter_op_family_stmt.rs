use pgls_query::protobuf::AlterOpFamilyStmt;

use super::prelude::*;

pub(super) fn emit_alter_op_family_stmt(e: &mut EventEmitter, n: &AlterOpFamilyStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::OPERATOR_KW);
    e.space();
    e.token(TokenKind::FAMILY_KW);
    e.space();

    if !n.opfamilyname.is_empty() {
        emit_dot_separated_list(e, &n.opfamilyname);
    }

    e.space();
    e.token(TokenKind::USING_KW);
    e.space();

    if !n.amname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.amname);
    }

    // Use indent and soft line break for ADD/DROP clause
    e.indent_start();
    e.line(crate::emitter::LineType::SoftOrSpace);

    // Start a group for the entire ADD/DROP clause to keep operator items together
    e.group_start();

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
                e.token(TokenKind::COMMA);
                e.line(crate::emitter::LineType::SoftOrSpace);
            }
            emit_node(e, item);
        }
    }

    e.group_end();
    e.indent_end();

    e.group_end();
}
