use pgt_query::protobuf::ReplicaIdentityStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_replica_identity_stmt(e: &mut EventEmitter, n: &ReplicaIdentityStmt) {
    e.group_start(GroupKind::ReplicaIdentityStmt);

    e.token(TokenKind::IDENT("REPLICA".to_string()));
    e.space();
    e.token(TokenKind::IDENT("IDENTITY".to_string()));
    e.space();

    // identity_type: 'd' = DEFAULT, 'f' = FULL, 'i' = USING INDEX, 'n' = NOTHING
    match n.identity_type.as_str() {
        "d" => {
            e.token(TokenKind::DEFAULT_KW);
        }
        "f" => {
            e.token(TokenKind::IDENT("FULL".to_string()));
        }
        "n" => {
            e.token(TokenKind::IDENT("NOTHING".to_string()));
        }
        "i" => {
            // USING INDEX index_name
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::INDEX_KW);
            if !n.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(n.name.clone()));
            }
        }
        _ => {
            // Fallback for unknown types
            e.token(TokenKind::IDENT(format!("UNKNOWN_{}", n.identity_type)));
        }
    }

    e.group_end();
}
