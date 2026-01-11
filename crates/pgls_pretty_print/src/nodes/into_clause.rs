use pgls_query::protobuf::{IntoClause, OnCommitAction};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::{
    emit_node,
    node_list::emit_comma_separated_list,
    string::{emit_identifier_maybe_quoted, emit_string},
};

pub(super) fn emit_into_clause(e: &mut EventEmitter, n: &IntoClause) {
    e.group_start(GroupKind::IntoClause);

    e.token(TokenKind::INTO_KW);

    if let Some(ref rel) = n.rel {
        e.space();

        match rel.relpersistence.as_str() {
            "t" => {
                e.token(TokenKind::TEMPORARY_KW);
                e.space();
                e.token(TokenKind::TABLE_KW);
                e.space();
            }
            "u" => {
                e.token(TokenKind::UNLOGGED_KW);
                e.space();
                e.token(TokenKind::TABLE_KW);
                e.space();
            }
            _ => {}
        }

        super::emit_range_var(e, rel);

        if !n.col_names.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            if n.col_names.len() > 1 {
                e.indent_start();
                e.line(LineType::SoftOrSpace);
            }
            emit_comma_separated_list(e, &n.col_names, |node, e| {
                let ident = assert_node_variant!(String, node);
                emit_string(e, ident);
            });
            if n.col_names.len() > 1 {
                e.indent_end();
            }
            e.token(TokenKind::R_PAREN);
        }
    }

    if !n.table_space_name.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.table_space_name);
    }

    if !n.access_method.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.access_method);
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        if n.options.len() > 1 {
            e.indent_start();
            e.line(LineType::SoftOrSpace);
        }
        emit_comma_separated_list(e, &n.options, emit_node);
        if n.options.len() > 1 {
            e.indent_end();
        }
        e.token(TokenKind::R_PAREN);
    }

    match OnCommitAction::try_from(n.on_commit).unwrap_or(OnCommitAction::Undefined) {
        OnCommitAction::OncommitPreserveRows => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::COMMIT_KW);
            e.space();
            e.token(TokenKind::PRESERVE_KW);
            e.space();
            e.token(TokenKind::ROWS_KW);
        }
        OnCommitAction::OncommitDeleteRows => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::COMMIT_KW);
            e.space();
            e.token(TokenKind::DELETE_KW);
            e.space();
            e.token(TokenKind::ROWS_KW);
        }
        OnCommitAction::OncommitDrop => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::COMMIT_KW);
            e.space();
            e.token(TokenKind::DROP_KW);
        }
        OnCommitAction::OncommitNoop | OnCommitAction::Undefined => {}
    }

    e.group_end();
}
