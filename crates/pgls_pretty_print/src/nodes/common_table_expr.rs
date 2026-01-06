use pgls_query::protobuf::{CommonTableExpr, CteMaterialize};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::delete_stmt::emit_delete_stmt_no_semicolon;
use super::insert_stmt::emit_insert_stmt_no_semicolon;
use super::merge_stmt::emit_merge_stmt_no_semicolon;
use super::node_list::emit_comma_separated_list;
use super::select_stmt::emit_select_stmt_no_semicolon;
use super::update_stmt::emit_update_stmt_no_semicolon;

pub(super) fn emit_common_table_expr(e: &mut EventEmitter, n: &CommonTableExpr) {
    e.group_start(GroupKind::CommonTableExpr);

    // CTE name
    e.token(TokenKind::IDENT(n.ctename.clone()));

    // Optional column aliases
    if !n.aliascolnames.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.aliascolnames, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::AS_KW);
    e.space();

    // Materialization hint (PostgreSQL 12+)
    match n.ctematerialized() {
        CteMaterialize::Always => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
        }
        CteMaterialize::Never => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
        }
        CteMaterialize::Default | CteMaterialize::CtematerializeUndefined => {
            // CTEMaterializeDefault/Undefined: omit hint to preserve planner choice
        }
    }

    // CTE query in parentheses
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);

    if let Some(ref query) = n.ctequery {
        // For CTEs, we don't want semicolons in the query
        // Check if it's a SelectStmt, MergeStmt, or InsertStmt and use the no-semicolon variant
        match &query.node {
            Some(pgls_query::NodeEnum::SelectStmt(select_stmt)) => {
                emit_select_stmt_no_semicolon(e, select_stmt);
            }
            Some(pgls_query::NodeEnum::MergeStmt(merge_stmt)) => {
                emit_merge_stmt_no_semicolon(e, merge_stmt);
            }
            Some(pgls_query::NodeEnum::InsertStmt(insert_stmt)) => {
                emit_insert_stmt_no_semicolon(e, insert_stmt);
            }
            Some(pgls_query::NodeEnum::UpdateStmt(update_stmt)) => {
                emit_update_stmt_no_semicolon(e, update_stmt);
            }
            Some(pgls_query::NodeEnum::DeleteStmt(delete_stmt)) => {
                emit_delete_stmt_no_semicolon(e, delete_stmt);
            }
            _ => {
                super::emit_node(query, e);
            }
        }
    }

    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);

    if let Some(ref search) = n.search_clause {
        e.line(LineType::SoftOrSpace);
        super::emit_ctesearch_clause(e, search);
    }

    if let Some(ref cycle) = n.cycle_clause {
        e.line(LineType::SoftOrSpace);
        super::emit_ctecycle_clause(e, cycle);
    }

    e.group_end();
}
