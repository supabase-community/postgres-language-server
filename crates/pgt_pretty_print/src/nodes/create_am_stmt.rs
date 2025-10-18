use pgt_query::protobuf::CreateAmStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_create_am_stmt(e: &mut EventEmitter, n: &CreateAmStmt) {
    e.group_start(GroupKind::CreateAmStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("ACCESS".to_string()));
    e.space();
    e.token(TokenKind::IDENT("METHOD".to_string()));

    if !n.amname.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.amname.clone()));
    }

    // TYPE
    // amtype is a single character: 'i' = INDEX, 't' = TABLE
    if !n.amtype.is_empty() {
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();
        let type_str = match n.amtype.as_str() {
            "i" => "INDEX",
            "t" => "TABLE",
            _ => &n.amtype, // fallback to original value
        };
        e.token(TokenKind::IDENT(type_str.to_string()));
    }

    // HANDLER
    if !n.handler_name.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("HANDLER".to_string()));
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.handler_name);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
