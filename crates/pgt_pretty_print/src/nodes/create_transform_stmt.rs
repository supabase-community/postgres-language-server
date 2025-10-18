use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CreateTransformStmt;

pub(super) fn emit_create_transform_stmt(e: &mut EventEmitter, n: &CreateTransformStmt) {
    e.group_start(GroupKind::CreateTransformStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::IDENT("REPLACE".to_string()));
        e.space();
    }

    e.token(TokenKind::IDENT("TRANSFORM".to_string()));
    e.space();
    e.token(TokenKind::FOR_KW);
    e.space();

    if let Some(ref type_name) = n.type_name {
        super::emit_type_name(e, type_name);
    }

    e.space();
    e.token(TokenKind::IDENT("LANGUAGE".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.lang.clone()));

    e.space();
    e.token(TokenKind::L_PAREN);

    let mut has_clause = false;
    if let Some(ref fromsql) = n.fromsql {
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::IDENT("SQL".to_string()));
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::IDENT("FUNCTION".to_string()));
        e.space();
        super::emit_object_with_args(e, fromsql);
        has_clause = true;
    }

    if let Some(ref tosql) = n.tosql {
        if has_clause {
            e.token(TokenKind::COMMA);
            e.space();
        }
        e.token(TokenKind::TO_KW);
        e.space();
        e.token(TokenKind::IDENT("SQL".to_string()));
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::IDENT("FUNCTION".to_string()));
        e.space();
        super::emit_object_with_args(e, tosql);
    }

    e.token(TokenKind::R_PAREN);
    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
