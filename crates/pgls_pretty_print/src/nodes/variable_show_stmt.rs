use pgls_query::protobuf::VariableShowStmt;

use super::prelude::*;

pub(super) fn emit_variable_show_stmt(e: &mut EventEmitter, n: &VariableShowStmt) {
    e.group_start();

    e.token(TokenKind::SHOW_KW);

    if !n.name.is_empty() {
        e.space();
        emit_variable_name(e, &n.name);
    }

    e.group_end();
}
