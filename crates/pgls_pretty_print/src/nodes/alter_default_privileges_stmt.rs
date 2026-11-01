use pgls_query::protobuf::AlterDefaultPrivilegesStmt;

use super::prelude::*;

pub(super) fn emit_alter_default_privileges_stmt(
    e: &mut EventEmitter,
    n: &AlterDefaultPrivilegesStmt,
) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DEFAULT_KW);
    e.space();
    e.token(TokenKind::PRIVILEGES_KW);

    // The clauses and the action continue the statement, so a broken one indents them.
    e.indent_start();

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
                            emit_grant_list(e, &list.items, emit_node);
                        } else {
                            emit_node(e, arg);
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
                            emit_grant_list(e, &list.items, emit_node);
                        } else {
                            emit_node(e, arg);
                        }
                    }
                }
                _ => {
                    // Unknown option - emit as-is
                    e.space();
                    emit_node(e, opt);
                }
            }
        }
    }

    // The actual GRANT/REVOKE statement
    if let Some(ref action) = n.action {
        e.line(LineType::SoftOrSpace);
        emit_node_enum(e, &NodeEnum::GrantStmt(action.clone()));
    }

    e.indent_end();

    e.group_end();
}
