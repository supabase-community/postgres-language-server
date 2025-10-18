use pgt_query::protobuf::RuleStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::emit_node;

pub(super) fn emit_rule_stmt(e: &mut EventEmitter, n: &RuleStmt) {
    e.group_start(GroupKind::RuleStmt);

    e.token(TokenKind::CREATE_KW);

    if n.replace {
        e.space();
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::REPLACE_KW);
    }

    e.space();
    e.token(TokenKind::RULE_KW);
    e.space();
    e.token(TokenKind::IDENT(n.rulename.clone()));

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();

    // Event: SELECT, UPDATE, DELETE, INSERT
    // CmdType enum: 0=UNKNOWN, 1=SELECT, 2=UPDATE, 3=INSERT, 4=DELETE
    match n.event {
        1 => e.token(TokenKind::SELECT_KW),
        2 => e.token(TokenKind::UPDATE_KW),
        3 => e.token(TokenKind::INSERT_KW),
        4 => e.token(TokenKind::DELETE_KW),
        _ => e.token(TokenKind::SELECT_KW), // default
    }

    e.space();
    e.token(TokenKind::TO_KW);
    e.space();

    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.space();
        e.token(TokenKind::WHERE_KW);
        e.space();
        emit_node(where_clause, e);
    }

    e.space();
    e.token(TokenKind::DO_KW);

    if n.instead {
        e.space();
        e.token(TokenKind::INSTEAD_KW);
    }

    e.space();

    // Actions - can be NOTHING or a list of statements
    if n.actions.is_empty() {
        e.token(TokenKind::NOTHING_KW);
    } else if n.actions.len() == 1 {
        emit_node(&n.actions[0], e);
    } else {
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_semicolon_separated_list(e, &n.actions, emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
