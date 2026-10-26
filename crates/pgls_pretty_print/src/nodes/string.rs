use pgls_query::protobuf::String as PgString;

use super::prelude::*;

pub(super) fn emit_string(e: &mut EventEmitter, n: &PgString) {
    e.group_start(GroupKind::String);
    emit_identifier_maybe_quoted(e, &n.sval);
    e.group_end();
}
