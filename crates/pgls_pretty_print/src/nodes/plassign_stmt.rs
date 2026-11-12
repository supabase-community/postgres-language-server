use pgls_query::protobuf::PlAssignStmt;

use super::prelude::*;

pub(super) fn emit_plassign_stmt(e: &mut EventEmitter, n: &PlAssignStmt) {
    e.group_start(GroupKind::PlassignStmt);

    if !n.name.is_empty() {
        emit_identifier_maybe_quoted(e, &n.name);
    }

    for indirection in &n.indirection {
        emit_node(indirection, e);
    }

    if let Some(ref select_stmt) = n.val {
        if !n.name.is_empty() || !n.indirection.is_empty() {
            e.space();
        }
        e.token(TokenKind::IDENT(":=".to_string()));
        e.space();
        super::emit_select_stmt(e, select_stmt);
    }

    e.group_end();
}
