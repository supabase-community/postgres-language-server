use pgls_query::protobuf::ColumnDef;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::{ListSeparatorSpacing, emit_comma_separated_list_with_spacing};

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
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        super::emit_node(raw_default, e);
    }

    // Add collation if specified
    if let Some(ref coll_clause) = n.coll_clause {
        e.line(LineType::SoftOrSpace);
        super::emit_collate_clause(e, coll_clause);
    }

    // Add column-level FDW OPTIONS clause (for foreign tables) - must come before constraints
    // Uses SoftOrSpace to allow break if needed
    if !n.fdwoptions.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::OPTIONS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list_with_spacing(
            e,
            &n.fdwoptions,
            ListSeparatorSpacing::Space,
            |opt, e| {
                let def_elem = assert_node_variant!(DefElem, opt);
                super::emit_options_def_elem(e, def_elem);
            },
        );
        e.token(TokenKind::R_PAREN);
    }

    // Add constraints if any
    // TODO: Handle IDENTITY constraints specially
    for constraint in &n.constraints {
        e.line(LineType::SoftOrSpace);
        super::emit_node(constraint, e);
    }

    e.group_end();
}
