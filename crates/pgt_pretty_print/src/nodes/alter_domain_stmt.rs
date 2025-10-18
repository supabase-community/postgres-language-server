use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterDomainStmt;

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_alter_domain_stmt(e: &mut EventEmitter, n: &AlterDomainStmt) {
    e.group_start(GroupKind::AlterDomainStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("DOMAIN".to_string()));
    e.space();

    if n.missing_ok {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    // Domain name (qualified)
    if !n.type_name.is_empty() {
        emit_dot_separated_list(e, &n.type_name);
    }

    // subtype field indicates the operation:
    // 'T' = SET DEFAULT, 'N' = DROP NOT NULL, 'O' = SET NOT NULL,
    // 'C' = ADD CONSTRAINT, 'X' = DROP CONSTRAINT, 'V' = VALIDATE CONSTRAINT
    e.space();
    match n.subtype.as_str() {
        "T" => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::DEFAULT_KW);
            if let Some(ref def) = n.def {
                e.space();
                super::emit_node(def, e);
            }
        }
        "N" => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::IDENT("NULL".to_string()));
        }
        "O" => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::IDENT("NULL".to_string()));
        }
        "C" => {
            e.token(TokenKind::ADD_KW);
            if let Some(ref def) = n.def {
                e.space();
                super::emit_node(def, e);
            }
        }
        "X" => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::IDENT("CONSTRAINT".to_string()));
            if n.missing_ok {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
            if !n.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(n.name.clone()));
            }
            // behavior: 0=Undefined, 1=DropRestrict, 2=DropCascade
            // Only emit CASCADE explicitly; RESTRICT is the default
            if n.behavior == 2 {
                e.space();
                e.token(TokenKind::IDENT("CASCADE".to_string()));
            }
        }
        "V" => {
            e.token(TokenKind::IDENT("VALIDATE".to_string()));
            e.space();
            e.token(TokenKind::IDENT("CONSTRAINT".to_string()));
            if !n.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(n.name.clone()));
            }
        }
        _ => {
            // Unknown subtype
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
