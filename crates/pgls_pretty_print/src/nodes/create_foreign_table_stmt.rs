use super::node_list::emit_comma_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::CreateForeignTableStmt;

pub(super) fn emit_create_foreign_table_stmt(e: &mut EventEmitter, n: &CreateForeignTableStmt) {
    e.group_start(GroupKind::CreateForeignTableStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::TABLE_KW);
    e.space();

    // Emit the base CREATE TABLE structure
    if let Some(ref base) = n.base_stmt {
        // Emit table name
        if let Some(ref relation) = base.relation {
            super::emit_range_var(e, relation);
        }

        // PARTITION OF parent_table
        if !base.inh_relations.is_empty() {
            e.line(crate::emitter::LineType::SoftOrSpace);
            e.token(TokenKind::PARTITION_KW);
            e.space();
            e.token(TokenKind::OF_KW);
            e.space();
            // Emit first inherited relation as parent table
            super::emit_node(&base.inh_relations[0], e);

            // Partition bound spec (DEFAULT or FOR VALUES ...)
            // For partition foreign tables, a bound spec is required
            if let Some(ref partbound) = base.partbound {
                e.line(crate::emitter::LineType::SoftOrSpace);
                super::emit_partition_bound_spec(e, partbound);
            } else {
                // Default partition when no partbound specified
                e.line(crate::emitter::LineType::SoftOrSpace);
                e.token(TokenKind::DEFAULT_KW);
            }
        } else {
            // Emit column definitions (always emit parentheses, even for empty list)
            e.space();
            e.token(TokenKind::L_PAREN);
            if !base.table_elts.is_empty() {
                e.indent_start();
                e.line(crate::emitter::LineType::SoftOrSpace);
                emit_comma_separated_list(e, &base.table_elts, super::emit_node);
                e.indent_end();
                e.line(crate::emitter::LineType::SoftOrSpace);
            }
            e.token(TokenKind::R_PAREN);
        }
    }

    // SERVER clause
    e.space();
    e.token(TokenKind::SERVER_KW);
    e.space();
    e.token(TokenKind::IDENT(n.servername.clone()));

    // OPTIONS clause
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::OPTIONS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
