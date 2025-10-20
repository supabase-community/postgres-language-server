use pgt_query::protobuf::AIndices;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_a_indices(e: &mut EventEmitter, n: &AIndices) {
    e.group_start(GroupKind::AIndices);

    e.token(TokenKind::L_BRACK);

    // Lower bound (if slice)
    if let Some(ref lidx) = n.lidx {
        super::emit_node(lidx, e);
    }

    // If upper bound exists, this is a slice [lower:upper]
    if n.uidx.is_some() {
        e.token(TokenKind::IDENT(":".to_string()));
    }

    // Upper bound
    if let Some(ref uidx) = n.uidx {
        super::emit_node(uidx, e);
    }

    e.token(TokenKind::R_BRACK);

    e.group_end();
}
