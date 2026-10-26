use pgls_query::protobuf::AlterForeignServerStmt;

use super::prelude::*;

pub(super) fn emit_alter_foreign_server_stmt(e: &mut EventEmitter, n: &AlterForeignServerStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    emit_keyword(e, "SERVER");
    e.space();

    if !n.servername.is_empty() {
        emit_identifier_maybe_quoted(e, &n.servername);
    }

    if n.has_version && !n.version.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_keyword(e, "VERSION");
        e.space();
        emit_single_quoted_str(e, &n.version);
        e.indent_end();
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_options_clause(e, &n.options);
        e.indent_end();
    }

    e.group_end();
}
