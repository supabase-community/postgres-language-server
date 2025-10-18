use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterEventTrigStmt;

pub(super) fn emit_alter_event_trig_stmt(e: &mut EventEmitter, n: &AlterEventTrigStmt) {
    e.group_start(GroupKind::AlterEventTrigStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("EVENT".to_string()));
    e.space();
    e.token(TokenKind::IDENT("TRIGGER".to_string()));
    e.space();

    if !n.trigname.is_empty() {
        e.token(TokenKind::IDENT(n.trigname.clone()));
    }

    e.space();

    // tgenabled: 'O'=ENABLE, 'D'=DISABLE, 'R'=ENABLE REPLICA, 'A'=ENABLE ALWAYS
    match n.tgenabled.as_str() {
        "O" => e.token(TokenKind::IDENT("ENABLE".to_string())),
        "D" => e.token(TokenKind::IDENT("DISABLE".to_string())),
        "R" => {
            e.token(TokenKind::IDENT("ENABLE".to_string()));
            e.space();
            e.token(TokenKind::IDENT("REPLICA".to_string()));
        }
        "A" => {
            e.token(TokenKind::IDENT("ENABLE".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ALWAYS".to_string()));
        }
        _ => {}
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
