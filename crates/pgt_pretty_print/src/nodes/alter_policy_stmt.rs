use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterPolicyStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_policy_stmt(e: &mut EventEmitter, n: &AlterPolicyStmt) {
    e.group_start(GroupKind::AlterPolicyStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("POLICY".to_string()));
    e.space();

    // Policy name
    if !n.policy_name.is_empty() {
        e.token(TokenKind::IDENT(n.policy_name.clone()));
    }

    // Table name
    if let Some(ref table) = n.table {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        super::emit_range_var(e, table);
    }

    // Optional: TO roles
    if !n.roles.is_empty() {
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();
        emit_comma_separated_list(e, &n.roles, super::emit_node);
    }

    // Optional: USING clause
    if let Some(ref qual) = n.qual {
        e.space();
        e.token(TokenKind::IDENT("USING".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(qual, e);
        e.token(TokenKind::R_PAREN);
    }

    // Optional: WITH CHECK clause
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
