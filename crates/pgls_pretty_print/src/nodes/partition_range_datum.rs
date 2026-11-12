use pgls_query::protobuf::{PartitionRangeDatum, PartitionRangeDatumKind};

use super::prelude::*;

pub(super) fn emit_partition_range_datum(e: &mut EventEmitter, n: &PartitionRangeDatum) {
    e.group_start();

    match n.kind() {
        PartitionRangeDatumKind::PartitionRangeDatumMinvalue => {
            e.token(TokenKind::MINVALUE_KW);
        }
        PartitionRangeDatumKind::PartitionRangeDatumMaxvalue => {
            e.token(TokenKind::MAXVALUE_KW);
        }
        PartitionRangeDatumKind::PartitionRangeDatumValue => {
            if let Some(ref value) = n.value {
                emit_node(e, value);
            }
        }
        PartitionRangeDatumKind::Undefined => {}
    }

    e.group_end();
}
