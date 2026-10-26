use pgls_query::protobuf::{RowCompareExpr, RowCompareType};

use super::prelude::*;

pub(super) fn emit_row_compare_expr(e: &mut EventEmitter, n: &RowCompareExpr) {
    e.group_start();

    emit_parenthesized_list(e, &n.largs, emit_node);

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

    emit_parenthesized_list(e, &n.rargs, emit_node);

    e.group_end();
}
