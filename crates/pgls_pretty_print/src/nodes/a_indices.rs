use pgls_query::protobuf::AIndices;

use super::prelude::*;

pub(super) fn emit_a_indices(e: &mut EventEmitter, n: &AIndices) {
    e.group_start(GroupKind::AIndices);

    e.token(TokenKind::L_BRACK);

    if n.is_slice {
        if let Some(ref lidx) = n.lidx {
            emit_node(lidx, e);
        }

        // Colon distinguishes slice syntax from single index lookups.
        e.token(TokenKind::IDENT(":".to_string()));

        if let Some(ref uidx) = n.uidx {
            emit_node(uidx, e);
        }
    } else {
        // Non-slice access should render whichever bound PostgreSQL stored.
        match (&n.lidx, &n.uidx) {
            (Some(lidx), None) => emit_node(lidx, e),
            (None, Some(uidx)) => emit_node(uidx, e),
            (Some(lidx), Some(uidx)) => {
                debug_assert!(false, "AIndices with both bounds but is_slice = false");
                emit_node(lidx, e);
                emit_node(uidx, e);
            }
            (None, None) => {}
        }
    }

    e.token(TokenKind::R_BRACK);

    e.group_end();
}
