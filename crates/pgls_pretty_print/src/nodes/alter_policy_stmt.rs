use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterPolicyStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_policy_stmt(e: &mut EventEmitter, n: &AlterPolicyStmt) {
    e.group_start(GroupKind::AlterPolicyStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::POLICY_KW);
    e.space();

    // Policy name
    super::emit_identifier_maybe_quoted(e, &n.policy_name);

    // Table name
    if let Some(ref table) = n.table {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        super::emit_range_var(e, table);
    }

    // Optional: TO roles
    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::TO_KW);
        e.space();
        emit_comma_separated_list(e, &n.roles, super::emit_node);
    }

    // Optional: USING clause
    if let Some(ref qual) = n.qual {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        super::emit_parenthesized_condition(e, GroupKind::AlterPolicyStmt, qual);
    }

    // Optional: WITH CHECK clause
    if let Some(ref with_check) = n.with_check {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::CHECK_KW);
        e.space();
        super::emit_parenthesized_condition(e, GroupKind::AlterPolicyStmt, with_check);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
