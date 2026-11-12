use pgls_query::protobuf::CreatePolicyStmt;

use super::prelude::*;

pub(super) fn emit_create_policy_stmt(e: &mut EventEmitter, n: &CreatePolicyStmt) {
    e.group_start(GroupKind::CreatePolicyStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::POLICY_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.policy_name);

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::ON_KW);
    e.space();

    if let Some(ref table) = n.table {
        super::emit_range_var(e, table);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::AS_KW);
    e.space();
    if n.permissive {
        e.token(TokenKind::IDENT("permissive".to_string()));
    } else {
        e.token(TokenKind::IDENT("restrictive".to_string()));
    }

    // Command: SELECT, INSERT, UPDATE, DELETE, ALL
    if !n.cmd_name.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        let cmd_upper = n.cmd_name.to_uppercase();
        match cmd_upper.as_str() {
            "ALL" => e.token(TokenKind::ALL_KW),
            "SELECT" => e.token(TokenKind::SELECT_KW),
            "INSERT" => e.token(TokenKind::INSERT_KW),
            "UPDATE" => e.token(TokenKind::UPDATE_KW),
            "DELETE" => e.token(TokenKind::DELETE_KW),
            _ => emit_keyword(e, &n.cmd_name),
        }
    }

    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::TO_KW);
        e.space();
        emit_comma_separated_list(e, &n.roles, |node, e| {
            if let Some(NodeEnum::RoleSpec(role)) = &node.node {
                super::emit_role_spec(e, role);
            }
        });
    }

    if let Some(ref qual) = n.qual {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_parenthesized_condition(e, GroupKind::CreatePolicyStmt, qual);
    }

    if let Some(ref with_check) = n.with_check {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::CHECK_KW);
        e.space();
        emit_parenthesized_condition(e, GroupKind::CreatePolicyStmt, with_check);
    }

    e.group_end();
}
