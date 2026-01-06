use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::JsonTablePath;

pub(super) fn emit_json_table_path(e: &mut EventEmitter, n: &JsonTablePath) {
    e.group_start(GroupKind::JsonTablePath);

    if n.name.is_empty() {
        super::emit_identifier(e, "jsonpath");
    } else {
        super::emit_identifier_maybe_quoted(e, &n.name);
    }

    e.group_end();
}
