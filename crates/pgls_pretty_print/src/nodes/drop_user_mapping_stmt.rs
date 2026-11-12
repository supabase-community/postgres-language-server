use pgls_query::protobuf::DropUserMappingStmt;

use super::prelude::*;

pub(super) fn emit_drop_user_mapping_stmt(e: &mut EventEmitter, n: &DropUserMappingStmt) {
    e.group_start();

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::USER_KW);
    e.space();
    e.token(TokenKind::MAPPING_KW);

    if n.missing_ok {
        e.space();
        emit_if_exists(e);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FOR_KW);
    e.space();

    if let Some(ref user) = n.user {
        super::emit_role_spec(e, user);
    }

    if !n.servername.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SERVER_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.servername);
    }

    e.group_end();
}
