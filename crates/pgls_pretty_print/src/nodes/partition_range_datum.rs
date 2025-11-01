use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{PartitionRangeDatum, PartitionRangeDatumKind};

pub(super) fn emit_partition_range_datum(e: &mut EventEmitter, n: &PartitionRangeDatum) {
    e.group_start(GroupKind::PartitionRangeDatum);

    match n.kind() {
        PartitionRangeDatumKind::PartitionRangeDatumMinvalue => {
            e.token(TokenKind::IDENT("MINVALUE".into()));
        }
        PartitionRangeDatumKind::PartitionRangeDatumMaxvalue => {
            e.token(TokenKind::IDENT("MAXVALUE".into()));
        }
        PartitionRangeDatumKind::PartitionRangeDatumValue => {
            if let Some(ref value) = n.value {
                super::emit_node(value, e);
            }
        }
        PartitionRangeDatumKind::Undefined => {}
    }

    e.group_end();
}
