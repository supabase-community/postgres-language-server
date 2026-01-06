use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterRoleSetStmt;

use super::role_spec::emit_role_spec;

pub(super) fn emit_alter_role_set_stmt(e: &mut EventEmitter, n: &AlterRoleSetStmt) {
    e.group_start(GroupKind::AlterRoleSetStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::ROLE_KW);
    e.space();

    if let Some(ref role) = n.role {
        emit_role_spec(e, role);
    }

    // Optional: IN DATABASE clause
    if !n.database.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IN_KW);
        e.space();
        e.token(TokenKind::DATABASE_KW);
        e.space();
        e.token(TokenKind::IDENT(n.database.clone()));
    }

    // The SET/RESET statement
    if let Some(ref setstmt) = n.setstmt {
        e.line(LineType::SoftOrSpace);
        super::emit_variable_set_stmt_no_semicolon(e, setstmt);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
