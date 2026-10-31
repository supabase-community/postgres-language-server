use pgls_query::protobuf::TriggerTransition;

use super::prelude::*;

pub(super) fn emit_trigger_transition(e: &mut EventEmitter, n: &TriggerTransition) {
    e.group_start(GroupKind::TriggerTransition);

    let mut label = if n.name.is_empty() {
        "transition".to_string()
    } else {
        format!("transition#{}", n.name)
    };

    label.push('[');
    label.push_str(if n.is_new { "new" } else { "old" });
    label.push(',');
    label.push_str(if n.is_table { "table" } else { "row" });
    label.push(']');

    emit_identifier(e, &label);

    e.group_end();
}
