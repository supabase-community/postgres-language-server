use pgls_query::protobuf::JsonTablePath;

use super::prelude::*;

pub(super) fn emit_json_table_path(e: &mut EventEmitter, n: &JsonTablePath) {
    e.group_start(GroupKind::JsonTablePath);

    if n.name.is_empty() {
        emit_identifier(e, "jsonpath");
    } else {
        emit_identifier_maybe_quoted(e, &n.name);
    }

    e.group_end();
}
