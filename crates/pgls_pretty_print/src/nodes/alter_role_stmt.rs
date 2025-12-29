use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::{NodeEnum, protobuf::AlterRoleStmt};

use super::def_elem::emit_role_option;
use super::role_spec::emit_role_spec;

pub(super) fn emit_alter_role_stmt(e: &mut EventEmitter, n: &AlterRoleStmt) {
    e.group_start(GroupKind::AlterRoleStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // action: 1 = ALTER ROLE, -1 = DROP ROLE (but DROP is handled separately)
    e.token(TokenKind::IDENT("ROLE".to_string()));
    e.space();

    if let Some(ref role) = n.role {
        emit_role_spec(e, role);
    }

    // Emit role options with line breaks
    if !n.options.is_empty() {
        e.indent_start();
        for opt in &n.options {
            if let Some(NodeEnum::DefElem(def)) = opt.node.as_ref() {
                e.line(LineType::SoftOrSpace);
                emit_role_option(e, def);
            }
        }
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
