use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::{InsertStmt, OverridingKind};

use super::node_list::emit_comma_separated_list;
use super::res_target::emit_column_name;

pub(super) fn emit_insert_stmt(e: &mut EventEmitter, n: &InsertStmt) {
    emit_insert_stmt_impl(e, n, true);
}

pub(super) fn emit_insert_stmt_no_semicolon(e: &mut EventEmitter, n: &InsertStmt) {
    emit_insert_stmt_impl(e, n, false);
}

fn emit_insert_stmt_impl(e: &mut EventEmitter, n: &InsertStmt, with_semicolon: bool) {
    e.group_start(GroupKind::InsertStmt);

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::INSERT_KW);
    e.space();
    e.token(TokenKind::INTO_KW);
    e.space();

    // Emit table name
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // Emit column list if present
    if !n.cols.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.cols, |node, e| {
            if let Some(pgt_query::NodeEnum::ResTarget(res_target)) = node.node.as_ref() {
                emit_column_name(e, res_target);
            } else {
                super::emit_node(node, e);
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    match n.r#override() {
        OverridingKind::OverridingUserValue => {
            e.space();
            e.token(TokenKind::OVERRIDING_KW);
            e.space();
            e.token(TokenKind::USER_KW);
            e.space();
            e.token(TokenKind::VALUE_KW);
        }
        OverridingKind::OverridingSystemValue => {
            e.space();
            e.token(TokenKind::OVERRIDING_KW);
            e.space();
            e.token(TokenKind::SYSTEM_KW);
            e.space();
            e.token(TokenKind::VALUE_KW);
        }
        OverridingKind::OverridingNotSet | OverridingKind::Undefined => {}
    }

    // Emit VALUES or SELECT or DEFAULT VALUES
    if let Some(ref select_stmt) = n.select_stmt {
        e.line(LineType::SoftOrSpace);
        // Use no-semicolon variant since INSERT will emit its own semicolon
        if let Some(pgt_query::NodeEnum::SelectStmt(stmt)) = select_stmt.node.as_ref() {
            super::emit_select_stmt_no_semicolon(e, stmt);
        } else {
            super::emit_node(select_stmt, e);
        }
    } else {
        // No select_stmt means DEFAULT VALUES
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        e.token(TokenKind::VALUES_KW);
    }

    // Emit ON CONFLICT clause if present
    if let Some(ref on_conflict) = n.on_conflict_clause {
        super::emit_on_conflict_clause(e, on_conflict);
    }

    if !n.returning_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        emit_comma_separated_list(e, &n.returning_list, super::emit_node);
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }

    e.group_end();
}
