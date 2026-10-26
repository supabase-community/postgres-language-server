use pgls_query::protobuf::CreateUserMappingStmt;

use super::prelude::*;

pub(super) fn emit_create_user_mapping_stmt(e: &mut EventEmitter, n: &CreateUserMappingStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::USER_KW);
    e.space();
    e.token(TokenKind::MAPPING_KW);

    if n.if_not_exists {
        e.space();
        emit_if_not_exists(e);
    }

    if let Some(ref user) = n.user {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        super::emit_role_spec(e, user);
    }

    if !n.servername.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SERVER_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.servername);
    }

    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::OPTIONS_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, |e, n| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
    }

    e.group_end();
}
