use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::CreateUserMappingStmt;

pub(super) fn emit_create_user_mapping_stmt(e: &mut EventEmitter, n: &CreateUserMappingStmt) {
    e.group_start(GroupKind::CreateUserMappingStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("USER".to_string()));
    e.space();
    e.token(TokenKind::IDENT("MAPPING".to_string()));

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if let Some(ref user) = n.user {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        super::emit_role_spec(e, user);
    }

    if !n.servername.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("SERVER".to_string()));
        e.space();
        e.token(TokenKind::IDENT(n.servername.clone()));
    }

    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("OPTIONS".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
