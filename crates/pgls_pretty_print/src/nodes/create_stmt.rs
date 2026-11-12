use pgls_query::protobuf::CreateStmt;
use pgls_query::protobuf::node::Node as NodeEnum;

use super::prelude::*;

/// Emit a column override for typed tables (OF typename).
/// These use the syntax: column_name WITH OPTIONS constraints
fn emit_typed_table_column_override(e: &mut EventEmitter, item: &pgls_query::protobuf::Node) {
    if let Some(NodeEnum::ColumnDef(col)) = &item.node {
        // Emit column name
        emit_identifier_maybe_quoted(e, &col.colname);

        // WITH OPTIONS keyword
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::OPTIONS_KW);

        // Emit constraints (e.g., GENERATED ALWAYS AS ... STORED)
        for constraint in &col.constraints {
            e.line(LineType::SoftOrSpace);
            emit_node(e, constraint);
        }
    } else {
        // Fall back to regular node emission
        emit_node(e, item);
    }
}

pub(super) fn emit_create_stmt(e: &mut EventEmitter, n: &CreateStmt) {
    e.group_start();

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
        emit_if_not_exists(e);
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
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PARTITION_KW);
        e.space();
        e.token(TokenKind::OF_KW);
        e.space();

        if !n.inh_relations.is_empty() {
            emit_comma_separated_list(e, &n.inh_relations, emit_node);
        }

        // Add constraints for partition tables
        let has_content = !n.table_elts.is_empty() || !n.constraints.is_empty();
        if has_content {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::L_PAREN);
            e.indent_start();
            e.line(LineType::Soft);

            let mut first = true;
            for item in &n.table_elts {
                if !first {
                    emit_list_comma(e, Some(item), ListSeparatorSpacing::SoftOrSpace);
                }
                emit_node(e, item);
                first = false;
            }
            for item in &n.constraints {
                if !first {
                    emit_list_comma(e, Some(item), ListSeparatorSpacing::SoftOrSpace);
                }
                emit_node(e, item);
                first = false;
            }

            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
        }

        // Add FOR VALUES clause
        if let Some(ref partbound) = n.partbound {
            e.line(LineType::SoftOrSpace);
            super::emit_partition_bound_spec(e, partbound);
        }

        // Add PARTITION BY for sub-partitioned tables
        if let Some(ref partspec) = n.partspec {
            e.line(LineType::SoftOrSpace);
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

        // Typed tables can have column overrides with WITH OPTIONS
        let has_content = !n.table_elts.is_empty() || !n.constraints.is_empty();
        if has_content {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::L_PAREN);
            e.indent_start();
            e.line(LineType::Soft);

            let mut first = true;
            for item in &n.table_elts {
                if !first {
                    emit_list_comma(e, Some(item), ListSeparatorSpacing::SoftOrSpace);
                }
                // For typed tables, columns need WITH OPTIONS syntax
                emit_typed_table_column_override(e, item);
                first = false;
            }
            for item in &n.constraints {
                if !first {
                    emit_list_comma(e, Some(item), ListSeparatorSpacing::SoftOrSpace);
                }
                emit_node(e, item);
                first = false;
            }

            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
        }
    } else {
        // Regular table with columns and constraints
        let has_content = !n.table_elts.is_empty() || !n.constraints.is_empty();
        let multiple_items = n.table_elts.len() + n.constraints.len() > 1;

        e.space();
        e.group_start();
        e.token(TokenKind::L_PAREN);
        // Force break for tables with multiple columns for readability. A table without columns
        // keeps `()` together.
        if multiple_items {
            e.line(LineType::Hard);
        } else if has_content {
            e.line(LineType::Soft);
        }

        if has_content {
            e.indent_start();

            let separator = if multiple_items {
                ListSeparatorSpacing::Hard
            } else {
                ListSeparatorSpacing::SoftOrSpace
            };
            let mut first = true;
            for item in &n.table_elts {
                if !first {
                    emit_list_comma(e, Some(item), separator);
                }
                emit_node(e, item);
                first = false;
            }
            for item in &n.constraints {
                if !first {
                    emit_list_comma(e, Some(item), separator);
                }
                emit_node(e, item);
                first = false;
            }

            e.indent_end();
        }

        if multiple_items {
            e.line(LineType::Hard);
        } else if has_content {
            e.line(LineType::Soft);
        }
        e.token(TokenKind::R_PAREN);
        e.group_end();

        // Add INHERITS clause for regular inheritance
        if !n.inh_relations.is_empty() && !is_partition_table {
            e.line(LineType::Hard);
            e.token(TokenKind::INHERITS_KW);
            e.space();
            emit_parenthesized_list(e, &n.inh_relations, emit_node);
        }

        // Add PARTITION BY clause for regular partitioned tables
        if let Some(ref partspec) = n.partspec {
            e.line(LineType::Hard);
            super::emit_partition_spec(e, partspec);
        }
    }

    // Add USING clause if specified (for table access method)
    if !n.access_method.is_empty() {
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.access_method);
    }

    // Add WITH options if specified - use SoftOrSpace to break if needed
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.group_start();
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.options, emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
        e.group_end();
    }

    // Add ON COMMIT clause if specified
    // OncommitNoop = 1 should not emit anything
    if n.oncommit > 1 {
        e.line(LineType::SoftOrSpace);
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
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.tablespacename);
    }

    e.group_end();
}
