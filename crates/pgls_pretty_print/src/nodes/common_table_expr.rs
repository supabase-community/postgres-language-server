use pgls_query::protobuf::{CommonTableExpr, CteMaterialize};

use super::prelude::*;

pub(super) fn emit_common_table_expr(e: &mut EventEmitter, n: &CommonTableExpr) {
    e.group_start();

    // CTE name
    emit_identifier_maybe_quoted(e, &n.ctename);

    // Optional column aliases
    if !n.aliascolnames.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.aliascolnames, emit_node);
    }

    // `name as (` stays on one line; a body that doesn't fit breaks after the parenthesis.
    e.space();
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
        emit_node(e, query);
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
