//! Lists of nodes and the separators between their items.

use pgls_query::Node;

use crate::emitter::{EventEmitter, LineType};
use crate::nodes::emit_node;
use crate::{CommaStyle, Layout, TokenKind};

/// Where a list may break between two items.
#[derive(Clone, Copy, Default)]
pub(crate) enum ListSeparatorSpacing {
    /// Breaks when the enclosing group breaks, and is a space otherwise.
    #[default]
    SoftOrSpace,
    /// Always a space, never a break.
    Space,
    /// Always breaks.
    Hard,
    /// Breaks only before an item that doesn't fit on the line, so each line takes as many items
    /// as fit.
    Fill,
}

/// Emits the comma before `next`, honouring `commaStyle`. With trailing commas the break follows
/// the comma: `a,\nb`. With leading commas it comes first, `a\n, b`, and an own-line comment written
/// before `next` stays before the comma rather than following it.
///
/// A comment written between an item and its comma, `a /* c */, b`, stays before the comma.
pub(crate) fn emit_list_comma(
    e: &mut EventEmitter,
    next: Option<&Node>,
    spacing: ListSeparatorSpacing,
) {
    // The comments lead the first node of `next`, which is not the node's own location when that
    // points into it, such as the operator of `a + 1`.
    let next_location = next
        .and_then(|node| node.node.as_ref())
        .and_then(crate::comments::first_node_location);
    if let Some(location) = next_location {
        e.take_comments_before_keyword_at(location, ",");
    }

    if matches!(e.config().comma_style, CommaStyle::Leading) {
        if let Some(location) = next_location {
            e.take_own_line_leading_comments_at(location);
        }

        // The break opportunity sits before the comma, so a broken list reads "\n, column" while
        // a single line one still reads "a, b".
        match spacing {
            ListSeparatorSpacing::SoftOrSpace => e.line(LineType::Soft),
            ListSeparatorSpacing::Space => {}
            ListSeparatorSpacing::Hard => e.line(LineType::Hard),
            ListSeparatorSpacing::Fill => e.line(LineType::FillNoSpace),
        }
        e.token(TokenKind::COMMA);
        e.space();
    } else {
        e.token(TokenKind::COMMA);
        match spacing {
            ListSeparatorSpacing::SoftOrSpace => e.line(LineType::SoftOrSpace),
            ListSeparatorSpacing::Space => e.space(),
            ListSeparatorSpacing::Hard => e.line(LineType::Hard),
            ListSeparatorSpacing::Fill => e.line(LineType::Fill),
        }
    }
}

/// Emit a comma-separated list with configurable spacing behavior
pub(crate) fn emit_comma_separated_list_with_spacing<F>(
    e: &mut EventEmitter,
    nodes: &[Node],
    spacing: ListSeparatorSpacing,
    render: F,
) where
    F: Fn(&mut EventEmitter, &Node),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            emit_list_comma(e, Some(n), spacing);
        }
        render(e, n);
    }
}

/// Emit a comma-separated list (default: can wrap to newline)
pub(crate) fn emit_comma_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&mut EventEmitter, &Node),
{
    emit_comma_separated_list_with_spacing(e, nodes, ListSeparatorSpacing::SoftOrSpace, render);
}

/// Emits `(a, b, c)`: on one line when it fits, otherwise with each item on its own line, one level
/// deeper, and the closing parenthesis back on its own line.
pub(crate) fn emit_parenthesized_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&mut EventEmitter, &Node),
{
    e.group_start();
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);
    emit_comma_separated_list(e, nodes, render);
    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);
    e.group_end();
}

/// Emit a comma-separated list that packs as many items as possible on each line.
pub(crate) fn emit_fill_comma_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&mut EventEmitter, &Node),
{
    emit_comma_separated_list_with_spacing(e, nodes, ListSeparatorSpacing::Fill, render);
}

/// Emit a comma-separated list whose items always break in expanded layout.
pub(crate) fn emit_comma_separated_list_with_layout_break<F>(
    e: &mut EventEmitter,
    nodes: &[Node],
    render: F,
) where
    F: Fn(&mut EventEmitter, &Node),
{
    let spacing = match e.config().layout {
        Layout::Expanded => ListSeparatorSpacing::Hard,
        Layout::Fit => ListSeparatorSpacing::SoftOrSpace,
    };
    emit_comma_separated_list_with_spacing(e, nodes, spacing, render);
}

pub(crate) fn emit_dot_separated_list(e: &mut EventEmitter, nodes: &[Node]) {
    emit_dot_separated_list_with(e, nodes, emit_node);
}

pub(crate) fn emit_dot_separated_list_with<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&mut EventEmitter, &Node),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }
        render(e, n);
    }
}

pub(crate) fn emit_space_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&mut EventEmitter, &Node),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.line(LineType::SoftOrSpace);
        }
        render(e, n);
    }
}

pub(crate) fn emit_semicolon_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&mut EventEmitter, &Node),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::SEMICOLON);
            e.line(LineType::SoftOrSpace);
        }
        render(e, n);
    }
}

/// Emits the arguments of a planner operator node, whose operator is only known by its OID.
pub(crate) fn emit_args_as_sequence(e: &mut EventEmitter, args: &[Node], opno: u32) {
    if args.is_empty() {
        e.token(TokenKind::IDENT(format!("op#{opno}")));
        return;
    }

    if args.len() == 1 {
        e.token(TokenKind::IDENT(format!("op#{opno}")));
        e.space();
        emit_node(e, &args[0]);
        return;
    }

    emit_node(e, &args[0]);
    for arg in &args[1..] {
        e.space();
        e.token(TokenKind::IDENT(format!("op#{opno}")));
        e.space();
        emit_node(e, arg);
    }
}
