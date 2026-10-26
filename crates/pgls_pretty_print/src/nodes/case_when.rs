use pgls_query::protobuf::CaseWhen;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_case_when(e: &mut EventEmitter, n: &CaseWhen) {
    e.group_start(GroupKind::CaseWhen);

    e.token(TokenKind::WHEN_KW);

    // A condition that breaks, an AND chain for instance, continues one level deeper than WHEN, so
    // that its operators don't read as siblings of WHEN.
    if let Some(ref expr) = n.expr {
        e.space();
        e.indent_start();
        super::emit_node(expr, e);
        e.indent_end();
    }

    let expanded = matches!(e.config().layout, crate::Layout::Expanded);
    if expanded {
        e.indent_start();
    }
    if let Some(ref result) = n.result {
        super::emit_comments_before_keyword(e, "then", result);
    }
    super::emit_layout_break(e);
    e.token(TokenKind::THEN_KW);

    if let Some(ref result) = n.result {
        e.space();
        super::emit_node(result, e);
    }
    if expanded {
        e.indent_end();
    }

    e.group_end();
}
