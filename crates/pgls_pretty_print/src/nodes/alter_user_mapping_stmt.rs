use pgls_query::protobuf::AlterUserMappingStmt;

use super::prelude::*;

pub(super) fn emit_alter_user_mapping_stmt(e: &mut EventEmitter, n: &AlterUserMappingStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::USER_KW);
    e.space();
    e.token(TokenKind::MAPPING_KW);
    e.space();
    e.token(TokenKind::FOR_KW);
    e.space();

    // User
    if let Some(ref user) = n.user {
        super::emit_role_spec(e, user);
    }

    // Server
    e.space();
    e.token(TokenKind::SERVER_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.servername);

    // Options
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        e.token(TokenKind::OPTIONS_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, |e, n| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.indent_end();
    }

    e.group_end();
}
