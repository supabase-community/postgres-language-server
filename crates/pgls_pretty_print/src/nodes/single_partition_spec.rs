use pgls_query::protobuf::SinglePartitionSpec;

use super::prelude::*;

pub(super) fn emit_single_partition_spec(e: &mut EventEmitter, _n: &SinglePartitionSpec) {
    e.group_start(GroupKind::SinglePartitionSpec);
    e.group_end();
}
