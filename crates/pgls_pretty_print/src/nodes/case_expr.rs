use pgls_query::protobuf::CaseExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_case_expr(e: &mut EventEmitter, n: &CaseExpr) {
    e.group_start(GroupKind::CaseExpr);

    e.token(TokenKind::CASE_KW);

    // Optional test expression (for simple CASE)
    if let Some(ref arg) = n.arg {
        e.space();
        super::emit_node(arg, e);
    }

    // Indent WHEN/ELSE clauses inside CASE
    e.indent_start();

    // WHEN clauses
    let expanded = matches!(e.config().layout, crate::Layout::Expanded);

    for (index, when_clause) in n.args.iter().enumerate() {
        if expanded && index == 0 {
            // `CASE WHEN` is the visual header of a searched CASE. Keeping it together gives the
            // following condition/THEN/ELSE breaks a stable indentation anchor.
            e.space();
        } else if expanded {
            super::emit_layout_break(e);
        } else {
            e.line(crate::emitter::LineType::SoftOrSpace);
        }
        super::emit_node(when_clause, e);
    }

    // ELSE clause
    if let Some(ref defresult) = n.defresult {
        super::emit_layout_break(e);
        e.token(TokenKind::ELSE_KW);
        e.space();
        super::emit_node(defresult, e);
    }

    e.indent_end();
    super::emit_layout_break(e);
    e.token(TokenKind::END_KW);

    e.group_end();
}
