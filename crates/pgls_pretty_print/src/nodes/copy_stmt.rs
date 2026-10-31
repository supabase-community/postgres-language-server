use pgls_query::protobuf::CopyStmt;

use super::prelude::*;

pub(super) fn emit_copy_stmt(e: &mut EventEmitter, n: &CopyStmt) {
    e.group_start();

    e.token(TokenKind::COPY_KW);
    e.space();

    // Table name or query
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);

        // Column list - wrap in group for compact formatting
        if !n.attlist.is_empty() {
            e.space();
            e.group_start();
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Soft);
            e.indent_start();
            emit_comma_separated_list(e, &n.attlist, emit_node);
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();
        }
    } else if let Some(ref query) = n.query {
        e.token(TokenKind::L_PAREN);
        emit_node(e, query);
        e.token(TokenKind::R_PAREN);
    }

    // TO or FROM
    e.line(LineType::SoftOrSpace);
    if n.is_from {
        e.token(TokenKind::FROM_KW);
    } else {
        e.token(TokenKind::TO_KW);
    }
    e.space();

    // PROGRAM or filename
    if n.is_program {
        emit_keyword(e, "PROGRAM");
        e.space();
    }

    if !n.filename.is_empty() {
        emit_single_quoted_str(e, &n.filename);
    } else {
        emit_keyword(e, "STDOUT");
    }

    // Options - wrap in group for compact formatting
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.group_start();
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.options, |e, n| {
            let def_elem = assert_node_variant!(DefElem, n);
            emit_copy_option(e, def_elem);
        });
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
        e.group_end();
    }

    // WHERE clause
    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        emit_clause_condition(e, where_clause);
    }

    e.group_end();
}

/// Emit COPY options with proper SQL syntax
/// Boolean options like HEADER, FREEZE, etc. when true are emitted without a value
fn emit_copy_option(e: &mut EventEmitter, n: &pgls_query::protobuf::DefElem) {
    e.group_start();

    emit_keyword(e, &n.defname);

    // Handle the value based on its type
    if let Some(ref arg) = n.arg
        && let Some(node_enum) = &arg.node
    {
        match node_enum {
            NodeEnum::Boolean(b) => {
                // For boolean options like HEADER, FREEZE, etc.:
                // - When true, emit just the option name (already done above)
                // - When false, we still need to emit false explicitly
                if !b.boolval {
                    e.space();
                    e.token(TokenKind::FALSE_KW);
                }
                // Note: when true, we don't emit any value - just the name is sufficient
            }
            NodeEnum::String(s) => {
                e.space();
                emit_string_literal(e, s);
            }
            NodeEnum::List(list) => {
                // For options like FORCE_QUOTE (column_list)
                e.space();
                emit_parenthesized_list(e, &list.items, emit_node);
            }
            _ => {
                e.space();
                emit_node(e, arg);
            }
        }
    }

    e.group_end();
}
