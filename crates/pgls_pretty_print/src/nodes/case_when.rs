use pgls_query::protobuf::CaseWhen;

use super::prelude::*;

pub(super) fn emit_case_when(e: &mut EventEmitter, n: &CaseWhen) {
    e.group_start();

    e.token(TokenKind::WHEN_KW);

    // A condition that breaks, an AND chain for instance, continues one level deeper than WHEN, so
    // that its operators don't read as siblings of WHEN.
    if let Some(ref expr) = n.expr {
        e.space();
        e.indent_start();
        emit_node(e, expr);
        e.indent_end();
    }

    let expanded = matches!(e.config().layout, crate::Layout::Expanded);
    if expanded {
        e.indent_start();
    }
    if let Some(ref result) = n.result {
        emit_comments_before_keyword(e, "then", result);
    }
    emit_layout_break(e);
    e.token(TokenKind::THEN_KW);

    if let Some(ref result) = n.result {
        e.space();
        emit_case_result(e, result);
    }
    if expanded {
        e.indent_end();
    }

    e.group_end();
}
