use pgls_query::protobuf::JsonReturning;

use super::prelude::*;

pub(super) fn emit_json_returning(e: &mut EventEmitter, returning: &JsonReturning) {
    e.group_start();

    let mut has_content = false;
    emit_json_returning_clause(e, returning, &mut has_content);

    e.group_end();
}
