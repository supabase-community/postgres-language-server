use pgls_query::protobuf::PrepareStmt;

use super::prelude::*;

pub(super) fn emit_prepare_stmt(e: &mut EventEmitter, n: &PrepareStmt) {
    e.group_start();

    e.token(TokenKind::PREPARE_KW);

    // Statement name
    if !n.name.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.name);
    }

    // Argument types
    if !n.argtypes.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_parenthesized_list(e, &n.argtypes, emit_node);
    }

    // AS query
    if let Some(ref query) = n.query {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::AS_KW);
        e.space();
        emit_node(e, query);
    }

    e.group_end();
}
