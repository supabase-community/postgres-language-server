use pgls_query::protobuf::LoadStmt;

use super::prelude::*;

/// `LOAD file_name`, where the file name is a string, even an empty one:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L11368
pub(super) fn emit_load_stmt(e: &mut EventEmitter, n: &LoadStmt) {
    e.group_start();

    e.token(TokenKind::LOAD_KW);
    e.space();
    emit_single_quoted_str(e, &n.filename);

    e.group_end();
}
