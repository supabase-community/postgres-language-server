use pgls_query::Node;

use crate::emitter::{EventEmitter, LineType};
use crate::{CommaStyle, TokenKind};

/// Controls the spacing behavior after separators in list helpers
#[derive(Clone, Copy, Default)]
pub(super) enum ListSeparatorSpacing {
    /// Use `e.line(LineType::SoftOrSpace)` - can wrap to newline if needed (default)
    #[default]
    SoftOrSpace,
    /// Use `e.space()` - always a space, never wraps
    Space,
}

/// Emit a comma-separated list with configurable spacing behavior
pub(super) fn emit_comma_separated_list_with_spacing<F>(
    e: &mut EventEmitter,
    nodes: &[Node],
    spacing: ListSeparatorSpacing,
    render: F,
) where
    F: Fn(&Node, &mut EventEmitter),
{
    let leading = matches!(e.config().comma_style, CommaStyle::Leading);

    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            if leading {
                if let Some(location) = n
                    .node
                    .as_ref()
                    .and_then(|node| crate::codegen::node_location::node_location(&node.to_ref()))
                {
                    e.take_own_line_leading_comments_at(location);
                }

                // The break opportunity sits before the comma, so a broken list reads
                // "\n, column" while a single line one still reads "a, b".
                match spacing {
                    ListSeparatorSpacing::SoftOrSpace => e.line(LineType::Soft),
                    ListSeparatorSpacing::Space => {}
                }
                e.token(TokenKind::COMMA);
                e.space();
            } else {
                e.token(TokenKind::COMMA);
                match spacing {
                    ListSeparatorSpacing::SoftOrSpace => e.line(LineType::SoftOrSpace),
                    ListSeparatorSpacing::Space => e.space(),
                }
            }
        }
        render(n, e);
    }
}

/// Emit a comma-separated list (default: can wrap to newline)
pub(super) fn emit_comma_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    emit_comma_separated_list_with_spacing(e, nodes, ListSeparatorSpacing::SoftOrSpace, render);
}

/// Emit a comma-separated list that packs as many items as possible on each line.
pub(super) fn emit_fill_comma_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::Fill);
        }
        render(n, e);
    }
}

/// Emit a comma-separated list whose items always break in expanded layout.
pub(super) fn emit_comma_separated_list_with_layout_break<F>(
    e: &mut EventEmitter,
    nodes: &[Node],
    render: F,
) where
    F: Fn(&Node, &mut EventEmitter),
{
    let leading = matches!(e.config().comma_style, CommaStyle::Leading);

    for (index, node) in nodes.iter().enumerate() {
        if index > 0 {
            if leading {
                super::emit_layout_break(e);
                e.token(TokenKind::COMMA);
                e.space();
            } else {
                e.token(TokenKind::COMMA);
                super::emit_layout_break(e);
            }
        }
        render(node, e);
    }
}

pub(super) fn emit_dot_separated_list(e: &mut EventEmitter, nodes: &[Node]) {
    emit_dot_separated_list_with(e, nodes, super::emit_node);
}

pub(super) fn emit_dot_separated_list_with<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }
        render(n, e);
    }
}

#[allow(dead_code)]
pub(super) fn emit_keyword_separated_list<F>(
    e: &mut EventEmitter,
    nodes: &[Node],
    keyword: TokenKind,
    render: F,
) where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.space();
            e.token(keyword.clone());
            e.line(LineType::SoftOrSpace);
        }
        render(n, e);
    }
}

pub(super) fn emit_space_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.line(LineType::SoftOrSpace);
        }
        render(n, e);
    }
}

pub(super) fn emit_semicolon_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::SEMICOLON);
            e.line(LineType::SoftOrSpace);
        }
        render(n, e);
    }
}
