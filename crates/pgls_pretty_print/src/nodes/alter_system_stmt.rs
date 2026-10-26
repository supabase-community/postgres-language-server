use pgls_query::protobuf::AlterSystemStmt;

use super::prelude::*;

pub(super) fn emit_alter_system_stmt(e: &mut EventEmitter, n: &AlterSystemStmt) {
    e.group_start(GroupKind::AlterSystemStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::SYSTEM_KW);
    e.line(LineType::SoftOrSpace);

    // Emit the SET statement
    if let Some(ref setstmt) = n.setstmt {
        super::emit_variable_set_stmt(e, setstmt);
    }

    e.group_end();
}
