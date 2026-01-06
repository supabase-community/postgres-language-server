use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::{NodeEnum, protobuf::AlterDefaultPrivilegesStmt};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_default_privileges_stmt(
    e: &mut EventEmitter,
    n: &AlterDefaultPrivilegesStmt,
) {
    e.group_start(GroupKind::AlterDefaultPrivilegesStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DEFAULT_KW);
    e.space();
    e.token(TokenKind::IDENT("PRIVILEGES".to_string()));

    // Options can contain FOR ROLE/USER and IN SCHEMA clauses
    for opt in &n.options {
        if let Some(NodeEnum::DefElem(def)) = opt.node.as_ref() {
            match def.defname.as_str() {
                "roles" => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::FOR_KW);
                    e.space();
                    e.token(TokenKind::ROLE_KW);
                    e.space();
                    if let Some(ref arg) = def.arg {
                        if let Some(NodeEnum::List(list)) = arg.node.as_ref() {
                            emit_comma_separated_list(e, &list.items, super::emit_node);
                        } else {
                            super::emit_node(arg, e);
                        }
                    }
                }
                "schemas" => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::IN_KW);
                    e.space();
                    e.token(TokenKind::SCHEMA_KW);
                    e.space();
                    if let Some(ref arg) = def.arg {
                        if let Some(NodeEnum::List(list)) = arg.node.as_ref() {
                            emit_comma_separated_list(e, &list.items, super::emit_node);
                        } else {
                            super::emit_node(arg, e);
                        }
                    }
                }
                _ => {
                    // Unknown option - emit as-is
                    e.space();
                    super::emit_node(opt, e);
                }
            }
        }
    }

    // The actual GRANT/REVOKE statement
    if let Some(ref action) = n.action {
        e.line(LineType::SoftOrSpace);
        super::emit_node_enum(&NodeEnum::GrantStmt(action.clone()), e);
    }

    e.group_end();
}
