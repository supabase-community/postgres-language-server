use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::{CmdType, MergeMatchKind, MergeStmt, MergeWhenClause};

use super::emit_node;

pub(super) fn emit_merge_stmt(e: &mut EventEmitter, n: &MergeStmt) {
    emit_merge_stmt_impl(e, n, true);
}

pub(super) fn emit_merge_stmt_no_semicolon(e: &mut EventEmitter, n: &MergeStmt) {
    emit_merge_stmt_impl(e, n, false);
}

fn emit_merge_stmt_impl(e: &mut EventEmitter, n: &MergeStmt, with_semicolon: bool) {
    e.group_start(GroupKind::MergeStmt);

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::MERGE_KW);
    e.space();
    e.token(TokenKind::INTO_KW);
    e.space();

    // Target relation
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // USING clause
    if let Some(ref source) = n.source_relation {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_node(source, e);
    }

    // ON condition
    if let Some(ref condition) = n.join_condition {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        emit_node(condition, e);
    }

    // WHEN clauses
    for when_clause_node in &n.merge_when_clauses {
        let when_clause = assert_node_variant!(MergeWhenClause, when_clause_node);
        e.line(LineType::SoftOrSpace);
        emit_merge_when_clause(e, when_clause);
    }

    // RETURNING clause
    if !n.returning_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        super::node_list::emit_comma_separated_list(e, &n.returning_list, super::emit_node);
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }

    e.group_end();
}

fn emit_merge_when_clause(e: &mut EventEmitter, clause: &MergeWhenClause) {
    e.group_start(GroupKind::MergeWhenClause);

    e.token(TokenKind::WHEN_KW);
    e.space();

    match clause.match_kind() {
        MergeMatchKind::MergeWhenMatched => {
            e.token(TokenKind::MATCHED_KW);
        }
        MergeMatchKind::MergeWhenNotMatchedBySource => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::MATCHED_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            e.token(TokenKind::IDENT("SOURCE".to_string()));
        }
        MergeMatchKind::MergeWhenNotMatchedByTarget => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::MATCHED_KW);
            if clause.condition.is_none() {
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                e.token(TokenKind::IDENT("TARGET".to_string()));
            }
        }
        _ => {}
    }

    // AND condition
    if let Some(ref cond) = clause.condition {
        e.space();
        e.token(TokenKind::AND_KW);
        e.space();
        emit_node(cond, e);
    }

    e.space();
    e.token(TokenKind::THEN_KW);
    e.space();

    // Command (UPDATE, INSERT, DELETE, or DO NOTHING)
    match clause.command_type() {
        CmdType::CmdUpdate => {
            e.token(TokenKind::UPDATE_KW);
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            // Emit SET clauses
            super::node_list::emit_comma_separated_list(e, &clause.target_list, |node, e| {
                let res_target = assert_node_variant!(ResTarget, node);
                super::res_target::emit_set_clause(e, res_target);
            });
        }
        CmdType::CmdInsert => {
            e.token(TokenKind::INSERT_KW);

            // Column list (if target_list is not empty)
            if !clause.target_list.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                super::node_list::emit_comma_separated_list(e, &clause.target_list, |node, e| {
                    let res_target = assert_node_variant!(ResTarget, node);
                    // Just emit the column name for INSERT column list
                    if !res_target.name.is_empty() {
                        e.token(TokenKind::IDENT(res_target.name.clone()));
                    }
                });
                e.token(TokenKind::R_PAREN);
            }

            // VALUES clause
            if !clause.values.is_empty() {
                e.space();
                e.token(TokenKind::VALUES_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                super::node_list::emit_comma_separated_list(e, &clause.values, super::emit_node);
                e.token(TokenKind::R_PAREN);
            } else {
                // DEFAULT VALUES
                e.space();
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                e.token(TokenKind::VALUES_KW);
            }
        }
        CmdType::CmdDelete => {
            e.token(TokenKind::DELETE_KW);
        }
        CmdType::Undefined | CmdType::CmdUnknown => {
            // DO NOTHING
            e.token(TokenKind::DO_KW);
            e.space();
            e.token(TokenKind::IDENT("NOTHING".to_string()));
        }
        _ => {
            e.token(TokenKind::DO_KW);
            e.space();
            e.token(TokenKind::IDENT("NOTHING".to_string()));
        }
    }

    e.group_end();
}
