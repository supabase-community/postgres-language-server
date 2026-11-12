use pgls_query::protobuf::WindowClause;

use super::prelude::*;

pub(super) fn emit_window_clause(e: &mut EventEmitter, n: &WindowClause) {
    e.group_start();

    // Emit: name AS (window_spec)
    emit_identifier_maybe_quoted(e, &n.name);
    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    emit_window_spec(e, n);

    e.group_end();
}

/// Emit the window specification part: (PARTITION BY ... ORDER BY ... frame_clause)
fn emit_window_spec(e: &mut EventEmitter, n: &WindowClause) {
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if !n.refname.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_identifier_maybe_quoted(e, &n.refname);
        has_content = true;
    }

    if !n.partition_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PARTITION_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.partition_clause, |emitter, node| {
            emit_node(emitter, node)
        });
        has_content = true;
    }

    if !n.order_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.order_clause, emit_node);
        has_content = true;
    }

    if has_window_frame(n.frame_options) {
        e.line(LineType::SoftOrSpace);
        emit_window_frame(
            e,
            n.frame_options,
            n.start_offset.as_deref(),
            n.end_offset.as_deref(),
            SingleBoundBreak::AfterMode,
        );
        has_content = true;
    }

    if !has_content {
        // Preserve empty parentheses for OVER ()
        e.token(TokenKind::R_PAREN);
        return;
    }

    e.token(TokenKind::R_PAREN);
}
