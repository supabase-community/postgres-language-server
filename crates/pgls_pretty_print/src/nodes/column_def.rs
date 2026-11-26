use pgls_query::protobuf::ColumnDef;

use super::prelude::*;

pub(super) fn emit_column_def(e: &mut EventEmitter, n: &ColumnDef) {
    e.group_start();

    // Column name (quote if necessary for special characters or keywords)
    emit_identifier_maybe_quoted(e, &n.colname);

    // Add type name
    if let Some(ref typename) = n.type_name {
        e.space();
        super::emit_type_name(e, typename);
    }

    // A broken column definition continues one level deeper, so that its clauses cannot be mistaken
    // for the next column.
    e.indent_start();

    // Add storage clause if specified
    if !n.storage_name.is_empty() {
        e.space();
        e.token(TokenKind::STORAGE_KW);
        e.space();
        e.token(TokenKind::IDENT(n.storage_name.clone()));
    }

    // Add compression clause if specified
    if !n.compression.is_empty() {
        e.space();
        e.token(TokenKind::COMPRESSION_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.compression);
    }

    // Storage, compression and the FDW options come in this order before the other clauses:
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L3814
    if !n.fdwoptions.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_options_clause(e, &n.fdwoptions);
    }

    // Add NOT NULL constraint if specified
    if n.is_not_null {
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
    }

    // Add DEFAULT clause if specified
    if let Some(ref raw_default) = n.raw_default {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        emit_node(e, raw_default);
    }

    // Add collation if specified
    if let Some(ref coll_clause) = n.coll_clause {
        e.line(LineType::SoftOrSpace);
        super::emit_collate_clause(e, coll_clause);
    }

    // Add constraints if any
    // TODO: Handle IDENTITY constraints specially
    for constraint in &n.constraints {
        e.line(LineType::SoftOrSpace);
        emit_node(e, constraint);
    }

    e.indent_end();
    e.group_end();
}
