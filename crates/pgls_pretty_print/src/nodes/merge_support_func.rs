use pgls_query::protobuf::MergeSupportFunc;

use super::prelude::*;

pub(super) fn emit_merge_support_func(e: &mut EventEmitter, n: &MergeSupportFunc) {
    e.group_start();

    emit_identifier(e, &format!("mergesupport#{}", n.msftype));

    if n.msfcollid != 0 {
        e.space();
        emit_identifier(e, &format!("coll#{}", n.msfcollid));
    }

    e.group_end();
}
