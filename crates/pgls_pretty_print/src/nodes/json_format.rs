use pgls_query::protobuf::JsonFormat;

use super::prelude::*;

pub(super) fn emit_json_format(e: &mut EventEmitter, format: &JsonFormat) {
    e.group_start();

    emit_json_format_after(e, format, Prefix::None);

    e.group_end();
}
