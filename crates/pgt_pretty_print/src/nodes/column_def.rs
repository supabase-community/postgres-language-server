use pgt_query::protobuf::ColumnDef;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_column_def(e: &mut EventEmitter, n: &ColumnDef) {
    e.group_start(GroupKind::ColumnDef);

    // Column name (quote if necessary for special characters or keywords)
    super::emit_identifier_maybe_quoted(e, &n.colname);

    // Add type name
    if let Some(ref typename) = n.type_name {
        e.space();
        super::emit_type_name(e, typename);
    }

    // Add compression clause if specified
    if !n.compression.is_empty() {
        e.space();
        e.token(TokenKind::COMPRESSION_KW);
        e.space();
        e.token(TokenKind::IDENT(n.compression.clone()));
    }

    // Add storage clause if specified
    if !n.storage_name.is_empty() {
        e.space();
        e.token(TokenKind::STORAGE_KW);
        e.space();
        e.token(TokenKind::IDENT(n.storage_name.clone()));
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
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        super::emit_node(raw_default, e);
    }

    // Add collation if specified
    // TODO: Implement CollateClause emission
    // if let Some(ref coll_clause) = n.coll_clause {
    //     e.space();
    //     super::emit_node(coll_clause, e);
    // }

    // Add constraints if any
    // TODO: Handle IDENTITY constraints specially
    for constraint in &n.constraints {
        e.space();
        super::emit_node(constraint, e);
    }

    e.group_end();
}
