use pgt_query::protobuf::DropUserMappingStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_drop_user_mapping_stmt(e: &mut EventEmitter, n: &DropUserMappingStmt) {
    e.group_start(GroupKind::DropUserMappingStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::USER_KW);
    e.space();
    e.token(TokenKind::MAPPING_KW);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    e.space();
    e.token(TokenKind::FOR_KW);
    e.space();

    if let Some(ref user) = n.user {
        super::emit_role_spec(e, user);
    }

    if !n.servername.is_empty() {
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(n.servername.clone()));
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
