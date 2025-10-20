use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::AlterSystemStmt;

pub(super) fn emit_alter_system_stmt(e: &mut EventEmitter, n: &AlterSystemStmt) {
    e.group_start(GroupKind::AlterSystemStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("SYSTEM".to_string()));
    e.space();

    // Emit the SET statement
    if let Some(ref setstmt) = n.setstmt {
        super::emit_variable_set_stmt(e, setstmt);
    }

    e.group_end();
}
