use pgls_query::protobuf::VariableShowStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_variable_show_stmt(e: &mut EventEmitter, n: &VariableShowStmt) {
    e.group_start(GroupKind::VariableShowStmt);

    e.token(TokenKind::SHOW_KW);

    if !n.name.is_empty() {
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.name);
    }

    e.group_end();
}
