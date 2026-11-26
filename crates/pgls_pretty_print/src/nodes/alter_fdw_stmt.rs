use pgls_query::protobuf::AlterFdwStmt;

use super::prelude::*;

pub(super) fn emit_alter_fdw_stmt(e: &mut EventEmitter, n: &AlterFdwStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::DATA_KW);
    e.space();
    e.token(TokenKind::WRAPPER_KW);
    e.space();

    if !n.fdwname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.fdwname);
    }

    emit_fdw_functions(e, &n.func_options);

    // OPTIONS clause
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_options_clause(e, &n.options);
        e.indent_end();
    }

    e.group_end();
}
