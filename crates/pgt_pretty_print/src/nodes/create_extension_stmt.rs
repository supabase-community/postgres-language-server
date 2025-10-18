use super::node_list::emit_comma_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CreateExtensionStmt;

pub(super) fn emit_create_extension_stmt(e: &mut EventEmitter, n: &CreateExtensionStmt) {
    e.group_start(GroupKind::CreateExtensionStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::EXTENSION_KW);
    e.space();

    if n.if_not_exists {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    e.token(TokenKind::IDENT(n.extname.clone()));

    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("WITH".to_string()));
        e.space();
        emit_comma_separated_list(e, &n.options, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
