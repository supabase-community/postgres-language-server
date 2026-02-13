use pgls_query::protobuf::CreatedbStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_space_separated_list;

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
        emit_space_separated_list(e, &n.options, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
