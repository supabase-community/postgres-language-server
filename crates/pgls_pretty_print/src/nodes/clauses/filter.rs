//! The FILTER clause of an aggregate call, `count(*) filter (where ...)`, which the JSON
//! aggregates take too.

use crate::nodes::prelude::*;

/// Emits `filter (where ...)`, on one line when it fits. Otherwise the condition goes on its own
/// lines inside the parentheses, like a policy's `using (...)`.
pub(crate) fn emit_filter_clause(e: &mut EventEmitter, filter: &Node) {
    e.token(TokenKind::FILTER_KW);
    e.space();
    e.group_start();
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);
    e.token(TokenKind::WHERE_KW);
    e.space();
    e.indent_start();
    emit_node(e, filter);
    e.indent_end();
    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);
    e.group_end();
}
