use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_dot_separated_list,
};
use pgt_query::protobuf::CreatePLangStmt;

pub(super) fn emit_create_plang_stmt(e: &mut EventEmitter, n: &CreatePLangStmt) {
    e.group_start(GroupKind::CreatePlangStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::IDENT("REPLACE".to_string()));
        e.space();
    }

    if n.pltrusted {
        e.token(TokenKind::IDENT("TRUSTED".to_string()));
        e.space();
    }

    e.token(TokenKind::IDENT("LANGUAGE".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.plname.clone()));

    if !n.plhandler.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("HANDLER".to_string()));
        e.space();
        emit_dot_separated_list(e, &n.plhandler);
    }

    if !n.plinline.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("INLINE".to_string()));
        e.space();
        emit_dot_separated_list(e, &n.plinline);
    }

    if !n.plvalidator.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("VALIDATOR".to_string()));
        e.space();
        emit_dot_separated_list(e, &n.plvalidator);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
