use pgls_query::protobuf::ImportForeignSchemaStmt;

use super::prelude::*;

pub(super) fn emit_import_foreign_schema_stmt(e: &mut EventEmitter, n: &ImportForeignSchemaStmt) {
    e.group_start();

    e.token(TokenKind::IMPORT_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::SCHEMA_KW);

    if !n.remote_schema.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.remote_schema);
    }

    // LIMIT TO / EXCEPT
    // list_type: 1=All, 2=LimitTo, 3=Except
    if !n.table_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        if n.list_type == 2 {
            // LIMIT TO
            e.token(TokenKind::LIMIT_KW);
            e.space();
            e.token(TokenKind::TO_KW);
        } else {
            // EXCEPT (3)
            e.token(TokenKind::EXCEPT_KW);
        }
        e.space();
        emit_parenthesized_list(e, &n.table_list, emit_node);
    }

    // FROM SERVER
    if !n.server_name.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.server_name);
    }

    // INTO schema
    if !n.local_schema.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::INTO_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.local_schema);
    }

    // OPTIONS
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_options_clause(e, &n.options);
    }

    e.group_end();
}
