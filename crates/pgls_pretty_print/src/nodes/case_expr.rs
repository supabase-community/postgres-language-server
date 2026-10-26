use pgls_query::protobuf::CaseExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_case_expr(e: &mut EventEmitter, n: &CaseExpr) {
    e.group_start(GroupKind::CaseExpr);

    let expanded = matches!(e.config().layout, crate::Layout::Expanded);
    if expanded {
        e.align_start();
    }
    e.token(TokenKind::CASE_KW);

    // Optional test expression (for simple CASE)
    if let Some(ref arg) = n.arg {
        e.space();
        super::emit_node(arg, e);
    }

    // Indent WHEN/ELSE clauses inside CASE
    e.indent_start();

    // WHEN clauses
    for when_clause in &n.args {
        super::emit_layout_break(e);
        super::emit_node(when_clause, e);
    }

    // ELSE clause
    if let Some(ref defresult) = n.defresult {
        super::emit_comments_before_keyword(e, "else", defresult);
        super::emit_layout_break(e);
        e.token(TokenKind::ELSE_KW);
        e.space();
        super::emit_node(defresult, e);
    }

    e.indent_end();
    super::emit_layout_break(e);
    e.token(TokenKind::END_KW);

    if expanded {
        e.align_end();
    }
    e.group_end();
}
