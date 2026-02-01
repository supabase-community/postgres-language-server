use pgls_query::protobuf::CaseTestExpr;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_case_test_expr(e: &mut EventEmitter, n: &CaseTestExpr) {
    e.group_start(GroupKind::CaseTestExpr);

    let repr = format!("case_test#{}_{}", n.type_id, n.type_mod);
    super::emit_identifier(e, &repr);

    e.group_end();
}
