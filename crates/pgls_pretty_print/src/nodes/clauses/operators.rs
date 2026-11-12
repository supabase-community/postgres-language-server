//! Operator names: `+` and `OPERATOR(schema.+)` in expressions, `schema.+` in DDL.
//!
//! The operator of `x = ANY (...)` is written by `sub_link.rs`, which keeps only the symbol.

use crate::nodes::prelude::*;

/// Emits the operator of an expression: `+`, or `OPERATOR(schema.+)` when it is qualified.
pub(crate) fn emit_operator(e: &mut EventEmitter, name: &[Node]) {
    if name.len() > 1 {
        emit_qualified_operator(e, name);
    } else if let Some(first) = name.first() {
        emit_operator_part(e, first);
    }
}

fn emit_qualified_operator(e: &mut EventEmitter, name: &[Node]) {
    e.token(TokenKind::OPERATOR_KW);
    e.token(TokenKind::L_PAREN);

    for (idx, part) in name.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }
        emit_operator_part(e, part);
    }

    e.token(TokenKind::R_PAREN);
}

/// Emits one part of an operator name: the symbol as written, or the schema.
pub(crate) fn emit_operator_part(e: &mut EventEmitter, node: &Node) {
    match node.node.as_ref() {
        Some(NodeEnum::String(s)) => e.token(TokenKind::IDENT(s.sval.clone())),
        _ => emit_node(e, node),
    }
}

/// Emits the name of an operator in DDL, `schema.+`, without the `OPERATOR(...)` an expression
/// needs.
pub(crate) fn emit_operator_name(e: &mut EventEmitter, defnames: &[Node]) {
    for (idx, node) in defnames.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match node.node.as_ref() {
            Some(NodeEnum::String(s)) => {
                if idx == defnames.len() - 1 {
                    e.token(TokenKind::IDENT(s.sval.clone()));
                } else {
                    emit_identifier_maybe_quoted(e, &s.sval);
                }
            }
            _ => emit_node(e, node),
        }
    }
}
