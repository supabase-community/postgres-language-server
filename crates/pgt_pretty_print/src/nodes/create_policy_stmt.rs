use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::{NodeEnum, protobuf::CreatePolicyStmt};

pub(super) fn emit_create_policy_stmt(e: &mut EventEmitter, n: &CreatePolicyStmt) {
    e.group_start(GroupKind::CreatePolicyStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("POLICY".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.policy_name.clone()));

    e.space();
    e.token(TokenKind::ON_KW);
    e.space();

    if let Some(ref table) = n.table {
        super::emit_range_var(e, table);
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    if n.permissive {
        e.token(TokenKind::IDENT("PERMISSIVE".to_string()));
    } else {
        e.token(TokenKind::IDENT("RESTRICTIVE".to_string()));
    }

    // Command: SELECT, INSERT, UPDATE, DELETE, ALL
    if !n.cmd_name.is_empty() {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        let cmd_upper = n.cmd_name.to_uppercase();
        match cmd_upper.as_str() {
            "ALL" => e.token(TokenKind::ALL_KW),
            "SELECT" => e.token(TokenKind::SELECT_KW),
            "INSERT" => e.token(TokenKind::INSERT_KW),
            "UPDATE" => e.token(TokenKind::UPDATE_KW),
            "DELETE" => e.token(TokenKind::DELETE_KW),
            _ => e.token(TokenKind::IDENT(cmd_upper)),
        }
    }

    if !n.roles.is_empty() {
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();
        emit_comma_separated_list(e, &n.roles, |node, e| {
            if let Some(NodeEnum::RoleSpec(role)) = &node.node {
                super::emit_role_spec(e, role);
            }
        });
    }

    if let Some(ref qual) = n.qual {
        e.space();
        e.token(TokenKind::IDENT("USING".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(qual, e);
        e.token(TokenKind::R_PAREN);
    }

    if let Some(ref with_check) = n.with_check {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::IDENT("CHECK".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(with_check, e);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
