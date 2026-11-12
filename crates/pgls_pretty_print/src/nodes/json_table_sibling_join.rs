use pgls_query::protobuf::JsonTableSiblingJoin;

use super::prelude::*;

pub(super) fn emit_json_table_sibling_join(e: &mut EventEmitter, n: &JsonTableSiblingJoin) {
    e.group_start(GroupKind::JsonTableSiblingJoin);

    emit_identifier(e, "jsonsiblingjoin");

    if let Some(plan) = n.plan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(plan, e);
    }

    if let Some(left) = n.lplan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(left, e);
    }

    if let Some(right) = n.rplan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(right, e);
    }

    e.group_end();
}
