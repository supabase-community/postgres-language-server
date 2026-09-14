use crate::{
    CastStyle, TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::{
    Node, NodeEnum,
    protobuf::{AExprKind, TypeCast},
};

pub(super) fn emit_type_cast(e: &mut EventEmitter, n: &TypeCast) {
    e.group_start(GroupKind::TypeCast);

    match e.config().cast_style {
        CastStyle::Operator => emit_operator_cast(e, n),
        CastStyle::Cast => emit_cast_call(e, n),
    }

    e.group_end();
}

fn emit_cast_call(e: &mut EventEmitter, n: &TypeCast) {
    e.token(TokenKind::CAST_KW);
    e.token(TokenKind::L_PAREN);

    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::AS_KW);
    e.space();

    if let Some(ref type_name) = n.type_name {
        super::emit_type_name(e, type_name);
    }

    e.token(TokenKind::R_PAREN);
}

fn emit_operator_cast(e: &mut EventEmitter, n: &TypeCast) {
    if let Some(ref arg) = n.arg {
        // `::` binds tighter than every infix operator, so anything that is not a self contained
        // primary expression has to be parenthesised: `a + b::int` would cast b alone.
        if needs_parentheses(arg) {
            e.token(TokenKind::L_PAREN);
            super::emit_node(arg, e);
            e.token(TokenKind::R_PAREN);
        } else {
            super::emit_node(arg, e);
        }
    }

    e.token(TokenKind::IDENT("::".to_string()));

    if let Some(ref type_name) = n.type_name {
        super::emit_type_name(e, type_name);
    }
}

/// Whitelist of self contained expressions that can carry a `::` without parentheses.
///
/// Anything absent from this list is parenthesised, which is always valid SQL. Listing the unsafe
/// kinds instead would fail open on any node nobody thought about.
fn needs_parentheses(node: &Node) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::AExpr(a_expr)) => a_expr.kind != AExprKind::AexprNullif as i32,
        Some(
            NodeEnum::AConst(_)
            | NodeEnum::ColumnRef(_)
            | NodeEnum::ParamRef(_)
            | NodeEnum::FuncCall(_)
            | NodeEnum::NullIfExpr(_)
            | NodeEnum::TypeCast(_)
            | NodeEnum::SubLink(_)
            | NodeEnum::CaseExpr(_)
            | NodeEnum::CoalesceExpr(_)
            | NodeEnum::MinMaxExpr(_)
            | NodeEnum::ArrayExpr(_)
            | NodeEnum::RowExpr(_)
            | NodeEnum::AIndirection(_)
            | NodeEnum::AArrayExpr(_),
        ) => false,
        _ => true,
    }
}
