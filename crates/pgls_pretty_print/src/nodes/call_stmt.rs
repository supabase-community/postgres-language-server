use pgls_query::protobuf::CallStmt;

use super::prelude::*;

pub(super) fn emit_call_stmt(e: &mut EventEmitter, n: &CallStmt) {
    e.group_start(GroupKind::CallStmt);

    e.token(TokenKind::CALL_KW);

    // Emit the function call
    if let Some(ref funccall) = n.funccall {
        e.space();
        super::emit_func_call(e, funccall);
    }

    e.group_end();
}
