use pgt_query::Node;

use crate::TokenKind;
use crate::emitter::{EventEmitter, LineType};

pub(super) fn emit_comma_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }
        render(n, e);
    }
}

pub(super) fn emit_dot_separated_list(e: &mut EventEmitter, nodes: &[Node]) {
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }
        super::emit_node(n, e);
    }
}

pub(super) fn emit_keyword_separated_list(
    e: &mut EventEmitter,
    nodes: &[Node],
    keyword: TokenKind,
) {
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.space();
            e.token(keyword.clone());
            e.line(LineType::SoftOrSpace);
        }
        super::emit_node(n, e);
    }
}

#[allow(dead_code)]
pub(super) fn emit_space_separated_list<F>(e: &mut EventEmitter, nodes: &[Node], render: F)
where
    F: Fn(&Node, &mut EventEmitter),
{
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            e.space();
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
