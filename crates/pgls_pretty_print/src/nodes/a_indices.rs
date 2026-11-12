use pgls_query::protobuf::AIndices;

use super::prelude::*;

pub(super) fn emit_a_indices(e: &mut EventEmitter, n: &AIndices) {
    e.group_start();

    e.token(TokenKind::L_BRACK);

    if n.is_slice {
        if let Some(ref lidx) = n.lidx {
            emit_node(e, lidx);
        }

        // Colon distinguishes slice syntax from single index lookups.
        e.token(TokenKind::IDENT(":".to_string()));

        if let Some(ref uidx) = n.uidx {
            emit_node(e, uidx);
        }
    } else {
        // Non-slice access should render whichever bound PostgreSQL stored.
        match (&n.lidx, &n.uidx) {
            (Some(lidx), None) => emit_node(e, lidx),
            (None, Some(uidx)) => emit_node(e, uidx),
            (Some(lidx), Some(uidx)) => {
                debug_assert!(false, "AIndices with both bounds but is_slice = false");
                emit_node(e, lidx);
                emit_node(e, uidx);
            }
            (None, None) => {}
        }
    }

    e.token(TokenKind::R_BRACK);

    e.group_end();
}
