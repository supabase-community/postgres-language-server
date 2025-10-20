use pgt_query::protobuf::{CommonTableExpr, CteMaterialize};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::merge_stmt::emit_merge_stmt_no_semicolon;
use super::node_list::emit_comma_separated_list;
use super::select_stmt::emit_select_stmt_no_semicolon;

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

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();

    // Materialization hint (PostgreSQL 12+)
    match n.ctematerialized() {
        CteMaterialize::Always => {
            e.token(TokenKind::IDENT("MATERIALIZED".to_string()));
            e.space();
        }
        CteMaterialize::Never => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::IDENT("MATERIALIZED".to_string()));
            e.space();
        }
        CteMaterialize::Default | CteMaterialize::CtematerializeUndefined => {
            // CTEMaterializeDefault/Undefined: omit hint to preserve planner choice
        }
    }

    // CTE query in parentheses
    e.token(TokenKind::L_PAREN);

    if let Some(ref query) = n.ctequery {
        // For CTEs, we don't want semicolons in the query
        // Check if it's a SelectStmt or MergeStmt and use the no-semicolon variant
        match &query.node {
            Some(pgt_query::NodeEnum::SelectStmt(select_stmt)) => {
                emit_select_stmt_no_semicolon(e, select_stmt);
            }
            Some(pgt_query::NodeEnum::MergeStmt(merge_stmt)) => {
                emit_merge_stmt_no_semicolon(e, merge_stmt);
            }
            _ => {
                super::emit_node(query, e);
            }
        }
    }

    e.token(TokenKind::R_PAREN);

    // TODO: SEARCH clause (PostgreSQL 14+)
    // TODO: CYCLE clause (PostgreSQL 14+)

    e.group_end();
}
