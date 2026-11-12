use pgls_query::protobuf::LoadStmt;

use super::prelude::*;

pub(super) fn emit_load_stmt(e: &mut EventEmitter, n: &LoadStmt) {
    e.group_start();

    e.token(TokenKind::LOAD_KW);

    if !n.filename.is_empty() {
        e.space();
        emit_single_quoted_str(e, &n.filename);
    }

    e.group_end();
}
