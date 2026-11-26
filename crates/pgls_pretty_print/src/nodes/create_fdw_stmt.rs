use pgls_query::protobuf::CreateFdwStmt;

use super::prelude::*;

pub(super) fn emit_create_fdw_stmt(e: &mut EventEmitter, n: &CreateFdwStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::DATA_KW);
    e.space();
    e.token(TokenKind::WRAPPER_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.fdwname);

    emit_fdw_functions(e, &n.func_options);

    // Options
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_options_clause(e, &n.options);
        e.indent_end();
    }

    e.group_end();
}
