use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_parenthesized_list,
};
use pgls_query::protobuf::{CompareType, RowCompareExpr};

pub(super) fn emit_row_compare_expr(e: &mut EventEmitter, n: &RowCompareExpr) {
    e.group_start(GroupKind::RowCompareExpr);

    emit_parenthesized_list(e, &n.largs, super::emit_node);

    e.space();

    // A planner node: the parser never produces it, so an unknown comparison is left as `?`.
    let op = match n.cmptype() {
        CompareType::CompareLt => "<",
        CompareType::CompareLe => "<=",
        CompareType::CompareEq => "=",
        CompareType::CompareGe => ">=",
        CompareType::CompareGt => ">",
        CompareType::CompareNe => "<>",
        CompareType::CompareOverlap => "&&",
        CompareType::CompareContainedBy => "<@",
        CompareType::CompareInvalid | CompareType::Undefined => "?",
    };
    e.token(TokenKind::IDENT(op.to_string()));

    e.space();

    emit_parenthesized_list(e, &n.rargs, super::emit_node);

    e.group_end();
}
