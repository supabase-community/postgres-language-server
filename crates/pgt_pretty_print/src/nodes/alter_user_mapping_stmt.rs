use super::node_list::emit_comma_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::AlterUserMappingStmt;

pub(super) fn emit_alter_user_mapping_stmt(e: &mut EventEmitter, n: &AlterUserMappingStmt) {
    e.group_start(GroupKind::AlterUserMappingStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("USER".to_string()));
    e.space();
    e.token(TokenKind::IDENT("MAPPING".to_string()));
    e.space();
    e.token(TokenKind::FOR_KW);
    e.space();

    // User
    if let Some(ref user) = n.user {
        super::emit_role_spec(e, user);
    }

    // Server
    e.space();
    e.token(TokenKind::IDENT("SERVER".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.servername.clone()));

    // Options
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        e.token(TokenKind::IDENT("OPTIONS".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.token(TokenKind::R_PAREN);
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
