use pgls_query::protobuf::ObjectWithArgs;

use super::prelude::*;

/// How the arguments of an object follow its name.
#[derive(Clone, Copy)]
pub(super) enum ObjectArguments {
    /// `(type, ...)`, and `()` for a function without arguments.
    Signature,
    /// Like `Signature`, except that an aggregate without arguments is written `(*)`, which means
    /// any arguments.
    AggregateSignature,
    /// No arguments, such as the operator of an operator class item.
    Omitted,
}

pub(super) fn emit_object_with_args(e: &mut EventEmitter, n: &ObjectWithArgs) {
    emit_object_with_args_as(e, n, ObjectArguments::Signature);
}

pub(super) fn emit_object_with_args_as(
    e: &mut EventEmitter,
    n: &ObjectWithArgs,
    arguments: ObjectArguments,
) {
    e.group_start();

    // Object name (qualified name)
    if !n.objname.is_empty() {
        emit_object_name(e, &n.objname);
    }

    if !matches!(arguments, ObjectArguments::Omitted) {
        let space_before_paren = needs_space_before_paren(n);
        // A function signature keeps its parameters as written, with their names and modes.
        // `objargs` only holds their types, and is all an operator has.
        let args = if n.objfuncargs.is_empty() {
            &n.objargs
        } else {
            &n.objfuncargs
        };
        if !args.is_empty() {
            if space_before_paren {
                e.space();
            }
            e.token(TokenKind::L_PAREN);
            if args.len() > 1 {
                e.indent_start();
                e.line(LineType::Soft);
                // For operators, NONE is represented by a Node with node: None
                emit_comma_separated_list(e, args, emit_objarg_or_none);
                e.indent_end();
            } else {
                emit_comma_separated_list(e, args, emit_objarg_or_none);
            }
            e.token(TokenKind::R_PAREN);
        } else if !n.args_unspecified {
            // Empty objargs with args_unspecified=false means:
            // - For aggregates: (*) meaning "any argument types"
            // - For functions: () meaning "no arguments"
            if space_before_paren {
                e.space();
            }
            e.token(TokenKind::L_PAREN);
            if matches!(arguments, ObjectArguments::AggregateSignature) {
                e.token(TokenKind::IDENT("*".to_string()));
            }
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}

/// Emit an operator argument, with NONE for missing types
fn emit_objarg_or_none(e: &mut EventEmitter, node: &pgls_query::Node) {
    if node.node.is_some() {
        emit_node(e, node);
    } else {
        // A Node with node: None represents NONE (no argument type)
        // Used for unary operators
        e.token(TokenKind::NONE_KW);
    }
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
            _ => emit_node(e, node),
        }
    }
}

fn is_operator_symbol(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| !c.is_ascii_alphanumeric() && c != '_')
}
