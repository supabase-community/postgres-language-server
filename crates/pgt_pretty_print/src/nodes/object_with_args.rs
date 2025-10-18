use pgt_query::{Node, NodeEnum, protobuf::ObjectWithArgs};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_object_with_args(e: &mut EventEmitter, n: &ObjectWithArgs) {
    emit_object_with_args_impl(e, n, true);
}

/// Emit ObjectWithArgs without parentheses (for operators in operator classes)
pub(super) fn emit_object_name_only(e: &mut EventEmitter, n: &ObjectWithArgs) {
    emit_object_with_args_impl(e, n, false);
}

fn emit_object_with_args_impl(e: &mut EventEmitter, n: &ObjectWithArgs, with_parens: bool) {
    e.group_start(GroupKind::ObjectWithArgs);

    // Object name (qualified name)
    if !n.objname.is_empty() {
        emit_object_name(e, &n.objname);
    }

    if with_parens {
        let space_before_paren = needs_space_before_paren(n);
        // Function arguments (for DROP FUNCTION, etc.)
        if !n.objargs.is_empty() {
            if space_before_paren {
                e.space();
            }
            e.token(TokenKind::L_PAREN);
            if n.objargs.len() > 1 {
                e.indent_start();
                e.line(LineType::Soft);
                emit_comma_separated_list(e, &n.objargs, super::emit_node);
                e.indent_end();
            } else {
                emit_comma_separated_list(e, &n.objargs, super::emit_node);
            }
            e.token(TokenKind::R_PAREN);
        } else if !n.args_unspecified {
            // Empty parens if args are specified as empty
            if space_before_paren {
                e.space();
            }
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}

fn needs_space_before_paren(n: &ObjectWithArgs) -> bool {
    n.objname
        .last()
        .and_then(|node| match node.node.as_ref() {
            Some(NodeEnum::String(s)) => Some(&s.sval),
            _ => None,
        })
        .map(|name| is_operator_symbol(name))
        .unwrap_or(false)
}

fn emit_object_name(e: &mut EventEmitter, items: &[Node]) {
    for (idx, node) in items.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match node.node.as_ref() {
            Some(NodeEnum::String(s)) if is_operator_symbol(&s.sval) => {
                e.token(TokenKind::IDENT(s.sval.clone()));
            }
            _ => super::emit_node(node, e),
        }
    }
}

fn is_operator_symbol(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| !c.is_ascii_alphanumeric() && c != '_')
}
