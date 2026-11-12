use pgls_query::protobuf::{CompareType, RowCompareExpr};

use super::prelude::*;

pub(super) fn emit_row_compare_expr(e: &mut EventEmitter, n: &RowCompareExpr) {
    e.group_start();

    emit_parenthesized_list(e, &n.largs, emit_node);

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

    emit_parenthesized_list(e, &n.rargs, emit_node);

    e.group_end();
}
