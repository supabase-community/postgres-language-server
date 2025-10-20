use super::{
    node_list::emit_comma_separated_list,
    string::{emit_keyword, emit_single_quoted_str},
};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CopyStmt;

pub(super) fn emit_copy_stmt(e: &mut EventEmitter, n: &CopyStmt) {
    e.group_start(GroupKind::CopyStmt);

    e.token(TokenKind::COPY_KW);
    e.space();

    // Table name or query
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);

        // Column list
        if !n.attlist.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.attlist, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
    } else if let Some(ref query) = n.query {
        e.token(TokenKind::L_PAREN);
        // Use no-semicolon variant for DML queries in COPY statement
        match &query.node {
            Some(pgt_query::NodeEnum::SelectStmt(stmt)) => {
                super::emit_select_stmt_no_semicolon(e, stmt);
            }
            Some(pgt_query::NodeEnum::InsertStmt(stmt)) => {
                super::emit_insert_stmt_no_semicolon(e, stmt);
            }
            Some(pgt_query::NodeEnum::UpdateStmt(stmt)) => {
                super::emit_update_stmt_no_semicolon(e, stmt);
            }
            Some(pgt_query::NodeEnum::DeleteStmt(stmt)) => {
                super::emit_delete_stmt_no_semicolon(e, stmt);
            }
            _ => {
                super::emit_node(query, e);
            }
        }
        e.token(TokenKind::R_PAREN);
    }

    // TO or FROM
    e.space();
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

    // Options
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.token(TokenKind::R_PAREN);
    }

    // WHERE clause
    if let Some(ref where_clause) = n.where_clause {
        e.space();
        e.token(TokenKind::WHERE_KW);
        e.space();
        super::emit_node(where_clause, e);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
