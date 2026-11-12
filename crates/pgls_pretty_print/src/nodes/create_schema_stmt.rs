use pgls_query::protobuf::CreateSchemaStmt;

use super::prelude::*;

pub(super) fn emit_create_schema_stmt(e: &mut EventEmitter, n: &CreateSchemaStmt) {
    e.group_start(GroupKind::CreateSchemaStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::SCHEMA_KW);

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.schemaname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.schemaname);
    }

    // AUTHORIZATION clause
    if let Some(ref authrole) = n.authrole {
        e.space();
        e.token(TokenKind::AUTHORIZATION_KW);
        e.space();
        super::emit_role_spec(e, authrole);
    }

    // Schema elements (nested CREATE statements).
    //
    // Per the PostgreSQL grammar (`OptSchemaEltList: OptSchemaEltList schema_stmt`),
    // elements are space-separated. No statement writes a `;`, so neither do they.
    if !n.schema_elts.is_empty() {
        // Soft line so the renderer can wrap when the schema name and the
        // first element don't fit on one line.
        e.line(LineType::SoftOrSpace);
        emit_space_separated_list(e, &n.schema_elts, emit_node);
    }

    e.group_end();
}
