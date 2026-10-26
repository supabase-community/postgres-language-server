//! Breaks and indentation shared by several nodes: call arguments, clause conditions and the
//! break between statement clauses.

use pgls_query::{Node, NodeEnum};

use crate::emitter::{EventEmitter, GroupKind, LineType};
use crate::nodes::{emit_node, emit_node_enum};
use crate::{ClauseBodyStyle, Layout, TokenKind};

use super::comments::emit_node_with;
use super::predicates::breaks_only_inside_a_subquery;

/// Opens the argument list of a call whose arguments the emitter writes itself: `(`, and when the
/// call doesn't fit, a break and one more level of indent.
pub(crate) fn open_call_arguments(e: &mut EventEmitter) {
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);
}

/// Closes an argument list opened with `open_call_arguments`, with `)` back on the call's indent.
pub(crate) fn close_call_arguments(e: &mut EventEmitter) {
    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);
}

pub(crate) fn emit_clause_condition(e: &mut EventEmitter, clause: &Node) {
    // Compact keeps the condition on the keyword line. The indent stays in both cases so that the
    // condition's own continuation lines, the second AND of a chain for instance, sit under it.
    match e.config().clause_body_style {
        ClauseBodyStyle::Compact => {
            e.space();
            emit_condition_on_keyword_line(e, clause);
        }
        ClauseBodyStyle::Break => {
            e.line(LineType::SoftOrSpace);
            e.indent_start();
            emit_node(clause, e);
            e.indent_end();
        }
    }
}

/// Emits a condition that starts on its keyword's line, `where` or `on`, indenting its
/// continuation lines.
///
/// A condition that only breaks inside the parentheses of a subquery, `exists (...)` or
/// `a = (...)`, has no continuation lines of its own. Indenting it would put the subquery two
/// levels in and the closing parenthesis one level in, so it keeps the keyword's indent and closes
/// under the keyword.
pub(crate) fn emit_condition_on_keyword_line(e: &mut EventEmitter, condition: &Node) {
    if breaks_only_inside_a_subquery(condition) {
        emit_node(condition, e);
    } else {
        e.indent_start();
        emit_node(condition, e);
        e.indent_end();
    }
}

/// Emits a condition wrapped in parentheses, such as the `USING (...)` of a policy.
///
/// It stays on the keyword line when it fits. Otherwise the condition breaks onto its own lines,
/// indented like a WHERE condition, and the closing parenthesis returns to the keyword's indent.
pub(crate) fn emit_parenthesized_condition(
    e: &mut EventEmitter,
    kind: GroupKind,
    condition: &Node,
) {
    emit_parenthesized_condition_with(e, kind, condition, emit_node_enum);
}

/// Like `emit_parenthesized_condition`, with `emit` emitting the condition itself.
pub(crate) fn emit_parenthesized_condition_with(
    e: &mut EventEmitter,
    kind: GroupKind,
    condition: &Node,
    emit: impl FnOnce(&NodeEnum, &mut EventEmitter),
) {
    e.group_start(kind);
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);
    emit_node_with(condition, e, emit);
    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);
    e.group_end();
}

/// Emits the break that separates a statement clause from the previous one.
///
/// In expanded layout it is a `Hard` line, which `try_single_line` refuses to collapse, so the
/// break propagates to every enclosing group and the whole statement opens up. That propagation is
/// why expanded layout needs no renderer change.
#[allow(dead_code)] // Consumed by the clause emitters introduced in task 3.
pub(crate) fn emit_layout_break(e: &mut EventEmitter) {
    match e.config().layout {
        Layout::Expanded => e.line(LineType::Hard),
        Layout::Fit => e.line(LineType::SoftOrSpace),
    }
}

#[cfg(test)]
mod tests {
    use crate::emitter::{EventEmitter, LayoutEvent, LineType};
    use crate::{FormatConfig, Layout};

    #[test]
    fn fit_layout_emits_a_soft_or_space_break() {
        let mut e = EventEmitter::new(FormatConfig::default());
        super::emit_layout_break(&mut e);
        assert_eq!(e.events, vec![LayoutEvent::Line(LineType::SoftOrSpace)]);
    }

    #[test]
    fn expanded_layout_emits_a_hard_break() {
        let config = FormatConfig {
            layout: Layout::Expanded,
            ..Default::default()
        };
        let mut e = EventEmitter::new(config);
        super::emit_layout_break(&mut e);
        assert_eq!(e.events, vec![LayoutEvent::Line(LineType::Hard)]);
    }
}
