use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::AlterEventTrigStmt;

pub(super) fn emit_alter_event_trig_stmt(e: &mut EventEmitter, n: &AlterEventTrigStmt) {
    e.group_start(GroupKind::AlterEventTrigStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::EVENT_KW);
    e.space();
    e.token(TokenKind::TRIGGER_KW);
    e.space();

    if !n.trigname.is_empty() {
        e.token(TokenKind::IDENT(n.trigname.clone()));
    }

    e.space();

    // tgenabled: 'O'=ENABLE, 'D'=DISABLE, 'R'=ENABLE REPLICA, 'A'=ENABLE ALWAYS
    match n.tgenabled.as_str() {
        "O" => e.token(TokenKind::ENABLE_KW),
        "D" => e.token(TokenKind::DISABLE_KW),
        "R" => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::REPLICA_KW);
        }
        "A" => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::ALWAYS_KW);
        }
        _ => {}
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
