use pgt_query::protobuf::CallStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_call_stmt(e: &mut EventEmitter, n: &CallStmt) {
    e.group_start(GroupKind::CallStmt);

    e.token(TokenKind::CALL_KW);

    // Emit the function call
    if let Some(ref funccall) = n.funccall {
        e.space();
        super::emit_func_call(e, funccall);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
