use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::WindowFuncRunCondition;

pub(super) fn emit_window_func_run_condition(e: &mut EventEmitter, n: &WindowFuncRunCondition) {
    e.group_start(GroupKind::WindowFuncRunCondition);

    let mut label = if n.opno != 0 {
        format!("winrun#{}", n.opno)
    } else {
        "winrun".to_string()
    };

    label.push('[');
    label.push_str(if n.wfunc_left { "left" } else { "right" });
    label.push(']');
    if n.inputcollid != 0 {
        label.push_str(&format!("@{}", n.inputcollid));
    }

    super::emit_identifier(e, &label);

    if let Some(arg) = n.arg.as_ref() {
        e.space();
        super::emit_node(arg, e);
    }

    e.group_end();
}
