use pgls_query::protobuf::ClusterStmt;

use super::prelude::*;

pub(super) fn emit_cluster_stmt(e: &mut EventEmitter, n: &ClusterStmt) {
    e.group_start();

    e.token(TokenKind::CLUSTER_KW);

    // VERBOSE option - check params
    if !n.params.is_empty() {
        e.space();
        e.token(TokenKind::VERBOSE_KW);
    }

    // Table name
    if let Some(ref relation) = n.relation {
        e.space();
        super::emit_range_var(e, relation);

        // Index name
        if !n.indexname.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::USING_KW);
            e.space();
            emit_identifier_maybe_quoted(e, &n.indexname);
        }
    }

    e.group_end();
}
