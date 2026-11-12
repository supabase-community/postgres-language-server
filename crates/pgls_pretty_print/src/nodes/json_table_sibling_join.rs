use pgls_query::protobuf::JsonTableSiblingJoin;

use super::prelude::*;

pub(super) fn emit_json_table_sibling_join(e: &mut EventEmitter, n: &JsonTableSiblingJoin) {
    e.group_start();

    emit_identifier(e, "jsonsiblingjoin");

    if let Some(plan) = n.plan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(e, plan);
    }

    if let Some(left) = n.lplan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(e, left);
    }

    if let Some(right) = n.rplan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(e, right);
    }

    e.group_end();
}
