use pgls_query::protobuf::AccessPriv;

use super::prelude::*;

/// Emits a privilege of GRANT, REVOKE or ALTER DEFAULT PRIVILEGES: `select`, `update (a, b)`, or
/// `all privileges (a, b)` for all privileges on some columns.
pub(super) fn emit_access_priv(e: &mut EventEmitter, n: &AccessPriv) {
    if n.priv_name.is_empty() {
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::PRIVILEGES_KW);
    } else {
        // Privilege names, `alter system` included, follow the keyword case.
        emit_keywords(e, &n.priv_name);
    }

    if !n.cols.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.cols, emit_node);
    }
}
