use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgt_query::protobuf::AlterForeignServerStmt;

use super::{
    node_list::emit_comma_separated_list,
    string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str},
};

pub(super) fn emit_alter_foreign_server_stmt(e: &mut EventEmitter, n: &AlterForeignServerStmt) {
    e.group_start(GroupKind::AlterForeignServerStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    emit_keyword(e, "SERVER");
    e.space();

    if !n.servername.is_empty() {
        emit_identifier_maybe_quoted(e, &n.servername);
    }

    if n.has_version && !n.version.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_keyword(e, "VERSION");
        e.space();
        emit_single_quoted_str(e, &n.version);
        e.indent_end();
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_keyword(e, "OPTIONS");
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.token(TokenKind::R_PAREN);
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
