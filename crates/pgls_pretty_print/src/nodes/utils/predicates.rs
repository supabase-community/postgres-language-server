//! Questions about the shape of a node that decide how it is printed.

use pgls_query::{Node, NodeEnum};

use crate::emitter::EventEmitter;

pub(crate) fn is_window_expression(node: &Node) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::FuncCall(call)) => call.over.is_some(),
        Some(NodeEnum::AExpr(expr)) => {
            expr.lexpr.as_deref().is_some_and(is_window_expression)
                || expr.rexpr.as_deref().is_some_and(is_window_expression)
        }
        _ => false,
    }
}

/// Whether a select item without an alias, which would continue on a line of its own, is written as
/// a name or keyword followed by
/// parentheses that hold everything that can break: a function call, `coalesce(...)`, `cast(...)`,
/// a subquery, ...
pub(crate) fn breaks_only_inside_its_parentheses(e: &EventEmitter, item: &Node) -> bool {
    let Some(NodeEnum::ResTarget(target)) = item.node.as_ref() else {
        return false;
    };
    if !target.name.is_empty() || !target.indirection.is_empty() {
        return false;
    }
    let Some(value) = target.val.as_deref() else {
        return false;
    };
    match value.node.as_ref() {
        // FILTER, WITHIN GROUP and OVER continue the call on lines of their own.
        Some(NodeEnum::FuncCall(call)) => {
            call.agg_filter.is_none() && call.over.is_none() && !call.agg_within_group
        }
        Some(NodeEnum::CoalesceExpr(_) | NodeEnum::MinMaxExpr(_) | NodeEnum::XmlSerialize(_)) => {
            true
        }
        Some(NodeEnum::TypeCast(_)) => matches!(e.config().cast_style, crate::CastStyle::Cast),
        Some(NodeEnum::SubLink(_)) => breaks_only_inside_a_subquery(value),
        _ => false,
    }
}

pub(crate) fn breaks_only_inside_a_subquery(node: &Node) -> bool {
    use pgls_query::protobuf::{AExprKind, BoolExprType, SubLinkType};

    match node.node.as_ref() {
        Some(NodeEnum::SubLink(sub_link)) => {
            sub_link.sub_link_type != SubLinkType::MultiexprSublink as i32
        }
        Some(NodeEnum::AExpr(expr)) => {
            expr.kind() == AExprKind::AexprOp
                && expr.lexpr.as_deref().is_some_and(is_simple_operand)
                && expr.rexpr.as_deref().is_some_and(|rexpr| {
                    matches!(
                        rexpr.node.as_ref(),
                        Some(NodeEnum::SubLink(sub_link))
                            if sub_link.sub_link_type == SubLinkType::ExprSublink as i32
                                || sub_link.sub_link_type == SubLinkType::ArraySublink as i32
                    )
                })
        }
        Some(NodeEnum::BoolExpr(expr)) => {
            expr.boolop() == BoolExprType::NotExpr
                && matches!(expr.args.as_slice(), [arg] if breaks_only_inside_a_subquery(arg))
        }
        _ => false,
    }
}

/// A column reference, parameter or constant, which never breaks.
fn is_simple_operand(node: &Node) -> bool {
    matches!(
        node.node.as_ref(),
        Some(NodeEnum::ColumnRef(_) | NodeEnum::ParamRef(_) | NodeEnum::AConst(_))
    )
}
