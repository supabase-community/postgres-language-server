use pgls_query::protobuf::PartitionCmd;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

pub(super) fn emit_partition_cmd(e: &mut EventEmitter, n: &PartitionCmd) {
    e.group_start(GroupKind::PartitionCmd);

    if let Some(ref name) = n.name {
        super::emit_range_var(e, name);
    }

    if n.concurrent {
        e.space();
        e.token(TokenKind::CONCURRENTLY_KW);
    }

    if let Some(ref bound) = n.bound {
        e.line(LineType::SoftOrSpace);
        super::emit_partition_bound_spec(e, bound);
    }

    e.group_end();
}
