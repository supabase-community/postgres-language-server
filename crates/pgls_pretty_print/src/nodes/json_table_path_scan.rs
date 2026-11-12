use pgls_query::protobuf::JsonTablePathScan;

use super::prelude::*;

pub(super) fn emit_json_table_path_scan(e: &mut EventEmitter, n: &JsonTablePathScan) {
    e.group_start();

    let mut label = String::from("jsonpathscan");
    if n.error_on_error {
        label.push_str("[error]");
    }
    if n.col_min != 0 || n.col_max != 0 {
        label.push_str(&format!("<{}:{}>", n.col_min, n.col_max));
    }
    emit_identifier(e, &label);

    if let Some(path) = n.path.as_ref() {
        e.space();
        super::emit_json_table_path(e, path);
    }

    if let Some(plan) = n.plan.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(e, plan);
    }

    if let Some(child) = n.child.as_ref() {
        e.line(LineType::SoftOrSpace);
        emit_node(e, child);
    }

    e.group_end();
}
