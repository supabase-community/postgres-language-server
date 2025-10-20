use pgt_query::protobuf::CreateStmt;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_create_stmt(e: &mut EventEmitter, n: &CreateStmt) {
    e.group_start(GroupKind::CreateStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    // Add TEMPORARY or UNLOGGED if specified
    if let Some(ref relation) = n.relation {
        match relation.relpersistence.as_str() {
            "t" => {
                e.token(TokenKind::TEMPORARY_KW);
                e.space();
            }
            "u" => {
                e.token(TokenKind::UNLOGGED_KW);
                e.space();
            }
            _ => {}
        }
    }

    e.token(TokenKind::TABLE_KW);

    // Add IF NOT EXISTS if specified
    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    // Add table name
    if let Some(ref relation) = n.relation {
        e.space();
        super::emit_range_var(e, relation);
    }

    // Handle different table types
    let is_partition_table = n.partbound.is_some() && !n.inh_relations.is_empty();
    let is_typed_table = n.of_typename.is_some();

    if is_partition_table {
        // PARTITION OF parent
        e.space();
        e.token(TokenKind::PARTITION_KW);
        e.space();
        e.token(TokenKind::OF_KW);
        e.space();

        if !n.inh_relations.is_empty() {
            emit_comma_separated_list(e, &n.inh_relations, super::emit_node);
        }

        // Add constraints for partition tables
        let has_content = !n.table_elts.is_empty() || !n.constraints.is_empty();
        if has_content {
            e.space();
            e.token(TokenKind::L_PAREN);
            e.indent_start();
            e.line(LineType::SoftOrSpace);

            let mut first = true;
            for item in &n.table_elts {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_node(item, e);
                first = false;
            }
            for item in &n.constraints {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_node(item, e);
                first = false;
            }

            e.indent_end();
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::R_PAREN);
        }

        // Add FOR VALUES clause
        if let Some(ref partbound) = n.partbound {
            e.space();
            super::emit_partition_bound_spec(e, partbound);
        }

        // Add PARTITION BY for sub-partitioned tables
        if let Some(ref partspec) = n.partspec {
            e.space();
            super::emit_partition_spec(e, partspec);
        }
    } else if is_typed_table {
        // OF typename
        e.space();
        e.token(TokenKind::OF_KW);
        e.space();
        if let Some(ref typename) = n.of_typename {
            super::emit_type_name(e, typename);
        }
    } else {
        // Regular table with columns and constraints
        let has_content = !n.table_elts.is_empty() || !n.constraints.is_empty();

        e.space();
        e.token(TokenKind::L_PAREN);

        if has_content {
            e.indent_start();
            e.line(LineType::SoftOrSpace);

            let mut first = true;
            for item in &n.table_elts {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_node(item, e);
                first = false;
            }
            for item in &n.constraints {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_node(item, e);
                first = false;
            }

            e.indent_end();
            e.line(LineType::SoftOrSpace);
        }

        e.token(TokenKind::R_PAREN);

        // Add INHERITS clause for regular inheritance
        if !n.inh_relations.is_empty() && !is_partition_table {
            e.space();
            e.token(TokenKind::INHERITS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.inh_relations, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }

        // Add PARTITION BY clause for regular partitioned tables
        if let Some(ref partspec) = n.partspec {
            e.space();
            super::emit_partition_spec(e, partspec);
        }
    }

    // Add USING clause if specified (for table access method)
    if !n.access_method.is_empty() {
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(n.access_method.clone()));
    }

    // Add WITH options if specified
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // Add ON COMMIT clause if specified
    // OncommitNoop = 1 should not emit anything
    if n.oncommit > 1 {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::COMMIT_KW);
        e.space();
        match n.oncommit {
            2 => {
                // ONCOMMIT_PRESERVE_ROWS
                e.token(TokenKind::PRESERVE_KW);
                e.space();
                e.token(TokenKind::ROWS_KW);
            }
            3 => {
                // ONCOMMIT_DELETE_ROWS
                e.token(TokenKind::DELETE_KW);
                e.space();
                e.token(TokenKind::ROWS_KW);
            }
            4 => {
                // ONCOMMIT_DROP
                e.token(TokenKind::DROP_KW);
            }
            _ => {}
        }
    }

    // Add tablespace if specified
    if !n.tablespacename.is_empty() {
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(n.tablespacename.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
