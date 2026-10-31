use pgls_query::protobuf::CreatePLangStmt;

use super::prelude::*;

pub(super) fn emit_create_plang_stmt(e: &mut EventEmitter, n: &CreatePLangStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        emit_or_replace(e);
        e.space();
    }

    if n.pltrusted {
        e.token(TokenKind::TRUSTED_KW);
        e.space();
    }

    e.token(TokenKind::LANGUAGE_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.plname);

    if !n.plhandler.is_empty() {
        e.space();
        e.token(TokenKind::HANDLER_KW);
        e.space();
        emit_dot_separated_list(e, &n.plhandler);
    }

    if !n.plinline.is_empty() {
        e.space();
        e.token(TokenKind::INLINE_KW);
        e.space();
        emit_dot_separated_list(e, &n.plinline);
    }

    if !n.plvalidator.is_empty() {
        e.space();
        e.token(TokenKind::VALIDATOR_KW);
        e.space();
        emit_dot_separated_list(e, &n.plvalidator);
    }

    e.group_end();
}
