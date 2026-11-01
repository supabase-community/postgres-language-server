use pgls_query::protobuf::CaseExpr;

use super::prelude::*;

pub(super) fn emit_case_expr(e: &mut EventEmitter, n: &CaseExpr) {
    emit_case_expr_impl(e, n, true);
}

/// Emits a branch result, `then <result>` or `else <result>`.
///
/// In expanded layout a CASE aligns its branches with the column it starts at. A CASE nested as a
/// result would start after `then `, so every level would move the next one right by the keyword
/// and two indents, and a few levels run past the line width. A nested CASE is therefore indented
/// from the line of its THEN or ELSE instead: its branches one level in, `end` under `then`.
pub(super) fn emit_case_result(e: &mut EventEmitter, result: &pgls_query::protobuf::Node) {
    emit_node_with(result, e, |node, e| match node {
        pgls_query::NodeEnum::CaseExpr(case) => emit_case_expr_impl(e, case, false),
        node => emit_node_enum(node, e),
    });
}

fn emit_case_expr_impl(e: &mut EventEmitter, n: &CaseExpr, align: bool) {
    e.group_start(GroupKind::CaseExpr);

    let expanded = matches!(e.config().layout, crate::Layout::Expanded);
    let align = expanded && align;
    if align {
        e.align_start();
    }
    e.token(TokenKind::CASE_KW);

    // Optional test expression (for simple CASE)
    if let Some(ref arg) = n.arg {
        e.space();
        emit_node(arg, e);
    }

    // Indent WHEN/ELSE clauses inside CASE
    e.indent_start();

    // WHEN clauses
    for when_clause in &n.args {
        emit_layout_break(e);
        emit_node(when_clause, e);
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
