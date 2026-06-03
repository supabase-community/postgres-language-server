use pgls_query::protobuf::DeallocateStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_deallocate_stmt(e: &mut EventEmitter, n: &DeallocateStmt) {
    e.group_start(GroupKind::DeallocateStmt);

    e.token(TokenKind::DEALLOCATE_KW);
    e.space();

    if n.name.is_empty() || n.name == "ALL" {
        e.token(TokenKind::ALL_KW);
    } else {
        super::emit_identifier_maybe_quoted(e, &n.name);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
