use super::{
    node_list::emit_comma_separated_list,
    string::{emit_keyword, emit_single_quoted_str},
};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::{NodeEnum, protobuf::CopyStmt};

pub(super) fn emit_copy_stmt(e: &mut EventEmitter, n: &CopyStmt) {
    e.group_start(GroupKind::CopyStmt);

    e.token(TokenKind::COPY_KW);
    e.space();

    // Table name or query
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);

        // Column list - wrap in group for compact formatting
        if !n.attlist.is_empty() {
            e.space();
            e.group_start(GroupKind::List);
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Soft);
            e.indent_start();
            emit_comma_separated_list(e, &n.attlist, super::emit_node);
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();
        }
    } else if let Some(ref query) = n.query {
        e.token(TokenKind::L_PAREN);
        // Use no-semicolon variant for DML queries in COPY statement
        match &query.node {
            Some(pgls_query::NodeEnum::SelectStmt(stmt)) => {
                super::emit_select_stmt_no_semicolon(e, stmt);
            }
            Some(pgls_query::NodeEnum::InsertStmt(stmt)) => {
                super::emit_insert_stmt_no_semicolon(e, stmt);
            }
            Some(pgls_query::NodeEnum::UpdateStmt(stmt)) => {
                super::emit_update_stmt_no_semicolon(e, stmt);
            }
            Some(pgls_query::NodeEnum::DeleteStmt(stmt)) => {
                super::emit_delete_stmt_no_semicolon(e, stmt);
            }
            Some(pgls_query::NodeEnum::MergeStmt(stmt)) => {
                super::emit_merge_stmt_no_semicolon(e, stmt);
            }
            _ => {
                super::emit_node(query, e);
            }
        }
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
        e.group_start(GroupKind::List);
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.options, |n, e| {
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
        super::emit_clause_condition(e, where_clause);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

/// Emit COPY options with proper SQL syntax
/// Boolean options like HEADER, FREEZE, etc. when true are emitted without a value
fn emit_copy_option(e: &mut EventEmitter, n: &pgls_query::protobuf::DefElem) {
    e.group_start(GroupKind::DefElem);

    // Emit the option name in uppercase
    let name_upper = n.defname.to_uppercase();
    e.token(TokenKind::IDENT(name_upper.clone()));

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
                super::emit_string_literal(e, s);
            }
            NodeEnum::List(list) => {
                // For options like FORCE_QUOTE (column_list)
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &list.items, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }
            _ => {
                e.space();
                super::emit_node(arg, e);
            }
        }
    }

    e.group_end();
}
