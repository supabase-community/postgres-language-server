use pgls_query::protobuf::JsonBehavior;

use super::prelude::*;

pub(super) fn emit_json_behavior(e: &mut EventEmitter, behavior: &JsonBehavior) {
    e.group_start();

    emit_json_behavior_keywords(e, behavior);

    if behavior.coerce {
        e.space();
        emit_identifier_maybe_quoted(e, "coerce");
    }

    e.group_end();
}
