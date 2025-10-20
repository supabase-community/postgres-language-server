use super::node_list::emit_comma_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::AlterTableSpaceOptionsStmt;

pub(super) fn emit_alter_tablespace_options_stmt(
    e: &mut EventEmitter,
    n: &AlterTableSpaceOptionsStmt,
) {
    e.group_start(GroupKind::AlterTableSpaceOptionsStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("TABLESPACE".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.tablespacename.clone()));

    e.space();
    if n.is_reset {
        e.token(TokenKind::IDENT("RESET".to_string()));
    } else {
        e.token(TokenKind::SET_KW);
    }

    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
