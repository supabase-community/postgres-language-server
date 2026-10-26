use pgls_query::protobuf::WindowDef;

use super::prelude::*;

/// What a window definition is written as.
#[derive(Clone, Copy)]
pub(super) enum WindowForm {
    /// The window of a call, after OVER: `(partition by ...)`, or the name of a window.
    Reference,
    /// A named window of the WINDOW clause: `w as (partition by ...)`.
    Named,
}

pub(super) fn emit_window_def(e: &mut EventEmitter, n: &WindowDef) {
    emit_window_def_as(e, n, WindowForm::Reference);
}

pub(super) fn emit_window_def_as(e: &mut EventEmitter, n: &WindowDef, form: WindowForm) {
    match form {
        WindowForm::Reference => emit_window_reference(e, n),
        WindowForm::Named => emit_named_window(e, n),
    }
}

fn emit_window_reference(e: &mut EventEmitter, n: &WindowDef) {
    // The callers of a window definition reach it directly rather than through emit_node, so this
    // is the only place that can emit the comments written before the OVER clause.
    emit_with_comments_at(e, n.location, |e| emit_window_def_tokens(e, n));
}

fn emit_window_def_tokens(e: &mut EventEmitter, n: &WindowDef) {
    // Simple reference to a named window
    if n.refname.is_empty()
        && n.partition_clause.is_empty()
        && n.order_clause.is_empty()
        && n.start_offset.is_none()
        && n.end_offset.is_none()
        && !n.name.is_empty()
    {
        emit_identifier_maybe_quoted(e, &n.name);
        return;
    }

    emit_window_spec(e, n);
}

fn emit_named_window(e: &mut EventEmitter, n: &WindowDef) {
    emit_identifier_maybe_quoted(e, &n.name);
    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    emit_window_spec(e, n);
}

fn emit_window_spec(e: &mut EventEmitter, n: &WindowDef) {
    e.token(TokenKind::L_PAREN);

    let has_content = !n.refname.is_empty()
        || !n.partition_clause.is_empty()
        || !n.order_clause.is_empty()
        || has_window_frame(n.frame_options);

    if !has_content {
        // Preserve empty parentheses for OVER ()
        e.token(TokenKind::R_PAREN);
        return;
    }

    e.line(LineType::Soft);
    e.indent_start();

    let mut first = true;

    if !n.refname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.refname);
        first = false;
    }

    if !n.partition_clause.is_empty() {
        if !first {
            e.line(LineType::SoftOrSpace);
        }
        e.token(TokenKind::PARTITION_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.partition_clause, |emitter, node| {
            emit_node(emitter, node)
        });
        first = false;
    }

    if !n.order_clause.is_empty() {
        if !first {
            e.line(LineType::SoftOrSpace);
        }
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.order_clause, emit_node);
        first = false;
    }

    if has_window_frame(n.frame_options) {
        if !first {
            e.line(LineType::SoftOrSpace);
        }
        emit_window_frame(
            e,
            n.frame_options,
            n.start_offset.as_deref(),
            n.end_offset.as_deref(),
            SingleBoundBreak::Never,
        );
    }

    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);
}
