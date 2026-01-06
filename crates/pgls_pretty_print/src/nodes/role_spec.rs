use pgls_query::protobuf::{RoleSpec, RoleSpecType};

use super::string::emit_identifier_maybe_quoted;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_role_spec(e: &mut EventEmitter, n: &RoleSpec) {
    e.group_start(GroupKind::RoleSpec);

    match n.roletype() {
        RoleSpecType::RolespecCstring => {
            if !n.rolename.is_empty() {
                emit_identifier_maybe_quoted(e, &n.rolename);
            }
        }
        RoleSpecType::RolespecCurrentUser => {
            e.token(TokenKind::CURRENT_USER_KW);
        }
        RoleSpecType::RolespecSessionUser => {
            e.token(TokenKind::SESSION_USER_KW);
        }
        RoleSpecType::RolespecCurrentRole => {
            e.token(TokenKind::CURRENT_ROLE_KW);
        }
        RoleSpecType::RolespecPublic => {
            e.token(TokenKind::IDENT("PUBLIC".to_string()));
        }
        RoleSpecType::Undefined => {
            if !n.rolename.is_empty() {
                emit_identifier_maybe_quoted(e, &n.rolename);
            }
        }
    }

    e.group_end();
}
