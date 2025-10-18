use super::node_list::{emit_comma_separated_list, emit_dot_separated_list};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::AlterTsDictionaryStmt;

pub(super) fn emit_alter_ts_dictionary_stmt(e: &mut EventEmitter, n: &AlterTsDictionaryStmt) {
    e.group_start(GroupKind::AlterTsdictionaryStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("TEXT".to_string()));
    e.space();
    e.token(TokenKind::IDENT("SEARCH".to_string()));
    e.space();
    e.token(TokenKind::IDENT("DICTIONARY".to_string()));
    e.space();

    // Dictionary name
    emit_dot_separated_list(e, &n.dictname);

    // Options
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
