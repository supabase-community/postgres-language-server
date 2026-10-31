use pgls_query::protobuf::TruncateStmt;

use super::prelude::*;

pub(super) fn emit_truncate_stmt(e: &mut EventEmitter, n: &TruncateStmt) {
    e.group_start();

    e.token(TokenKind::TRUNCATE_KW);

    if !n.relations.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.relations, emit_node);
    }

    // RESTART IDENTITY / CONTINUE IDENTITY
    if n.restart_seqs {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RESTART_KW);
        e.space();
        e.token(TokenKind::IDENTITY_KW);
    }

    emit_drop_behavior(e, n.behavior(), Separator::SoftOrSpace, Restrict::Omitted);

    e.group_end();
}
