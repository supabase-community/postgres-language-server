use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::SinglePartitionSpec;

pub(super) fn emit_single_partition_spec(e: &mut EventEmitter, _n: &SinglePartitionSpec) {
    e.group_start(GroupKind::SinglePartitionSpec);
    e.group_end();
}
