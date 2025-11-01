use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::JsonTablePathScan;

pub(super) fn emit_json_table_path_scan(e: &mut EventEmitter, n: &JsonTablePathScan) {
    e.group_start(GroupKind::JsonTablePathScan);

    let mut label = String::from("jsonpathscan");
    if n.error_on_error {
        label.push_str("[error]");
    }
    if n.col_min != 0 || n.col_max != 0 {
        label.push_str(&format!("<{}:{}>", n.col_min, n.col_max));
    }
    super::emit_identifier(e, &label);

    if let Some(path) = n.path.as_ref() {
        e.space();
        super::json_table_path::emit_json_table_path(e, path);
    }

    if let Some(plan) = n.plan.as_ref() {
        e.line(LineType::SoftOrSpace);
        super::emit_node(plan, e);
    }

    if let Some(child) = n.child.as_ref() {
        e.line(LineType::SoftOrSpace);
        super::emit_node(child, e);
    }

    e.group_end();
}
