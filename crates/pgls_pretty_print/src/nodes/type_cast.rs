use pgls_query::protobuf::TypeCast;

use crate::CastStyle;

use super::prelude::*;

pub(super) fn emit_type_cast(e: &mut EventEmitter, n: &TypeCast) {
    e.group_start(GroupKind::TypeCast);

    match e.config().cast_style {
        CastStyle::Operator => emit_operator_cast(e, n),
        CastStyle::Cast => emit_cast_call(e, n),
    }

    e.group_end();
}

/// Emits `CAST(expr AS type)` whatever the configured cast style.
///
/// A cast in a `FROM` clause is a table function, and only the `CAST(...)` spelling is accepted
/// there: `FROM (1 + 2)::int` is a syntax error.
pub(super) fn emit_type_cast_call(e: &mut EventEmitter, n: &TypeCast) {
    e.group_start(GroupKind::TypeCast);
    emit_cast_call(e, n);
    e.group_end();
}

/// Laid out like a function call: on one line when it fits, otherwise `expr as type` on its own
/// line one level in, broken before `as` only when it doesn't fit there either.
fn emit_cast_call(e: &mut EventEmitter, n: &TypeCast) {
    e.token(TokenKind::CAST_KW);
    open_call_arguments(e);
    e.group_start(GroupKind::List);

    if let Some(ref arg) = n.arg {
        emit_node(arg, e);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::AS_KW);
    e.space();

    if let Some(ref type_name) = n.type_name {
        super::emit_type_name(e, type_name);
    }

    e.group_end();
    close_call_arguments(e);
}

fn emit_operator_cast(e: &mut EventEmitter, n: &TypeCast) {
    if let Some(ref arg) = n.arg {
        // `::` binds tighter than every infix operator, so anything that is not a self contained
        // primary expression has to be parenthesised: `a + b::int` would cast b alone.
        if needs_parentheses(arg, Position::Postfix) {
            e.token(TokenKind::L_PAREN);
            emit_node(arg, e);
            e.token(TokenKind::R_PAREN);
        } else {
            emit_node(arg, e);
        }
    }

    e.token(TokenKind::IDENT("::".to_string()));

    if let Some(ref type_name) = n.type_name {
        super::emit_type_name(e, type_name);
    }
}
