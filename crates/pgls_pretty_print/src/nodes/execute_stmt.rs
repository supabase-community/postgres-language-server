use pgls_query::protobuf::ExecuteStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::emit_node;

pub(super) fn emit_execute_stmt(e: &mut EventEmitter, n: &ExecuteStmt) {
    e.group_start(GroupKind::ExecuteStmt);

    e.token(TokenKind::EXECUTE_KW);
    e.space();
    super::emit_identifier_maybe_quoted(e, &n.name);

    if !n.params.is_empty() {
        e.line(LineType::SoftOrSpace);
        super::node_list::emit_parenthesized_list(e, &n.params, emit_node);
    }

    e.group_end();
}
