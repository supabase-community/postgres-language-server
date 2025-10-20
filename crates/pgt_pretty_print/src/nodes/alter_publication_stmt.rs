use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterPublicationStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_publication_stmt(e: &mut EventEmitter, n: &AlterPublicationStmt) {
    e.group_start(GroupKind::AlterPublicationStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("PUBLICATION".to_string()));
    e.space();

    // Publication name
    if !n.pubname.is_empty() {
        e.token(TokenKind::IDENT(n.pubname.clone()));
    }

    // action: 0=Undefined, 1=AP_AddObjects, 2=AP_DropObjects, 3=AP_SetObjects
    match n.action {
        1 => {
            // ADD
            e.space();
            e.token(TokenKind::ADD_KW);
        }
        2 => {
            // DROP
            e.space();
            e.token(TokenKind::DROP_KW);
        }
        3 => {
            // SET
            e.space();
            e.token(TokenKind::SET_KW);
        }
        _ => {}
    }

    // Emit objects or FOR ALL TABLES
    if n.for_all_tables {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::IDENT("TABLES".to_string()));
    } else if !n.pubobjects.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.pubobjects, super::emit_node);
    }

    // Optional: WITH (options)
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
