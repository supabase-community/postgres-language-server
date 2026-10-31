use pgls_query::protobuf::CaseTestExpr;

use super::prelude::*;

pub(super) fn emit_case_test_expr(e: &mut EventEmitter, n: &CaseTestExpr) {
    e.group_start();

    let repr = format!("case_test#{}_{}", n.type_id, n.type_mod);
    emit_identifier(e, &repr);

    e.group_end();
}
