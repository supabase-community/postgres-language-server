use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::ClusterStmt;

pub(super) fn emit_cluster_stmt(e: &mut EventEmitter, n: &ClusterStmt) {
    e.group_start(GroupKind::ClusterStmt);

    e.token(TokenKind::IDENT("CLUSTER".to_string()));

    // VERBOSE option - check params
    if !n.params.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("VERBOSE".to_string()));
    }

    // Table name
    if let Some(ref relation) = n.relation {
        e.space();
        super::emit_range_var(e, relation);

        // Index name
        if !n.indexname.is_empty() {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::IDENT(n.indexname.clone()));
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
