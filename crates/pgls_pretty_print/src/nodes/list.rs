use pgls_query::protobuf::List;

use super::prelude::*;

pub(super) fn emit_list(e: &mut EventEmitter, n: &List) {
    e.group_start();

    emit_comma_separated_list(e, &n.items, emit_node);

    e.group_end();
}
