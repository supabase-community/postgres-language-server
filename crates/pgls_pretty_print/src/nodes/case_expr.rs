use pgls_query::protobuf::CaseExpr;

use super::prelude::*;

/// Where the branches of a CASE line up in expanded layout.
#[derive(Clone, Copy)]
pub(super) enum BranchPlacement {
    /// Under the column the CASE starts at.
    Aligned,
    /// One level in from the line the CASE starts on.
    Indented,
}

pub(super) fn emit_case_expr(e: &mut EventEmitter, n: &CaseExpr) {
    emit_case_expr_as(e, n, BranchPlacement::Aligned);
}

pub(super) fn emit_case_expr_as(e: &mut EventEmitter, n: &CaseExpr, branches: BranchPlacement) {
    e.group_start();

    let expanded = matches!(e.config().layout, crate::Layout::Expanded);
    let align = expanded && matches!(branches, BranchPlacement::Aligned);
    if align {
        e.align_start();
    }
    e.token(TokenKind::CASE_KW);

    // Optional test expression (for simple CASE)
    if let Some(ref arg) = n.arg {
        e.space();
        emit_node(e, arg);
    }

    // Indent WHEN/ELSE clauses inside CASE
    e.indent_start();

    // WHEN clauses
    for when_clause in &n.args {
        emit_layout_break(e);
        emit_node(e, when_clause);
    }

    // ELSE clause
    if let Some(ref defresult) = n.defresult {
        emit_comments_before_keyword(e, "else", defresult);
        emit_layout_break(e);
        e.token(TokenKind::ELSE_KW);
        e.space();
        emit_case_result(e, defresult);
    }

    e.indent_end();
    emit_layout_break(e);
    e.token(TokenKind::END_KW);

    if align {
        e.align_end();
    }
    e.group_end();
}
