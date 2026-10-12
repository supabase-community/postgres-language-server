use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_parenthesized_list,
};
use pgls_query::protobuf::{RowCompareExpr, RowCompareType};

pub(super) fn emit_row_compare_expr(e: &mut EventEmitter, n: &RowCompareExpr) {
    e.group_start(GroupKind::RowCompareExpr);

    emit_parenthesized_list(e, &n.largs, super::emit_node);

    e.space();

    let op = match n.rctype() {
        RowCompareType::RowcompareLt => "<",
        RowCompareType::RowcompareLe => "<=",
        RowCompareType::RowcompareEq => "=",
        RowCompareType::RowcompareGe => ">=",
        RowCompareType::RowcompareGt => ">",
        RowCompareType::RowcompareNe => "<>",
        RowCompareType::Undefined => {
            debug_assert!(false, "RowCompareExpr missing rctype");
            "?"
        }
    };
    e.token(TokenKind::IDENT(op.to_string()));

    e.space();

    emit_parenthesized_list(e, &n.rargs, super::emit_node);

    e.group_end();
}
