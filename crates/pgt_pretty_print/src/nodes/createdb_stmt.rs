use pgt_query::protobuf::CreatedbStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_createdb_stmt(e: &mut EventEmitter, n: &CreatedbStmt) {
    e.group_start(GroupKind::CreatedbStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);

    if !n.dbname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.dbname);
    }

    // Emit database options (WITH CONNECTION LIMIT, ENCODING, etc.)
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        for (i, opt) in n.options.iter().enumerate() {
            if i > 0 {
                e.space();
            }
            super::emit_node(opt, e);
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
