use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::{NodeEnum, protobuf::AlterRoleStmt};

use super::def_elem::emit_role_option;
use super::role_spec::emit_role_spec;

pub(super) fn emit_alter_role_stmt(e: &mut EventEmitter, n: &AlterRoleStmt) {
    e.group_start(GroupKind::AlterRoleStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Check if this is ALTER GROUP ... ADD/DROP USER (for group membership changes)
    // This is indicated by having a "rolemembers" option with a non-zero action
    let rolemembers_opt = n.options.iter().find(|opt| {
        matches!(
            opt.node.as_ref(),
            Some(NodeEnum::DefElem(def)) if def.defname == "rolemembers"
        )
    });
    let is_member_change = rolemembers_opt.is_some() && n.action != 0;

    if is_member_change {
        // This is ALTER GROUP ... ADD/DROP USER syntax (deprecated but still supported)
        e.token(TokenKind::IDENT("GROUP".to_string()));
        e.space();

        if let Some(ref role) = n.role {
            emit_role_spec(e, role);
        }

        e.line(LineType::SoftOrSpace);
        if n.action > 0 {
            e.token(TokenKind::IDENT("ADD".to_string()));
        } else {
            e.token(TokenKind::DROP_KW);
        }
        e.space();
        e.token(TokenKind::IDENT("USER".to_string()));

        // Emit the member list from the "rolemembers" option
        if let Some(opt) = rolemembers_opt {
            if let Some(NodeEnum::DefElem(def)) = opt.node.as_ref() {
                if let Some(ref arg) = def.arg {
                    e.space();
                    super::emit_node(arg, e);
                }
            }
        }
    } else {
        // Standard ALTER ROLE syntax
        e.token(TokenKind::ROLE_KW);
        e.space();

        if let Some(ref role) = n.role {
            emit_role_spec(e, role);
        }
        // Emit role options with line breaks for regular ALTER ROLE
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
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
