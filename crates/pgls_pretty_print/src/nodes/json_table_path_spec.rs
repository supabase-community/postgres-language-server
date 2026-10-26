use pgls_query::protobuf::JsonTablePathSpec;

use super::prelude::*;

pub(super) fn emit_json_table_path_spec(e: &mut EventEmitter, spec: &JsonTablePathSpec) {
    e.group_start();

    if let Some(string_node) = spec.string.as_ref() {
        emit_node(e, string_node);
    }

    if !spec.name.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &spec.name);
    }

    e.group_end();
}
