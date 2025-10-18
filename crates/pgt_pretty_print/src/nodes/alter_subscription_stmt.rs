use super::{
    node_list::emit_comma_separated_list,
    string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str},
};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::AlterSubscriptionStmt;

pub(super) fn emit_alter_subscription_stmt(e: &mut EventEmitter, n: &AlterSubscriptionStmt) {
    e.group_start(GroupKind::AlterSubscriptionStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    emit_keyword(e, "SUBSCRIPTION");
    e.space();
    emit_identifier_maybe_quoted(e, &n.subname);

    e.space();

    // Kind enum: 0=Undefined, 1=OPTIONS, 2=CONNECTION, 3=SET_PUBLICATION, 4=ADD_PUBLICATION, 5=DROP_PUBLICATION, 6=REFRESH, 7=ENABLED, 8=SKIP
    match n.kind {
        1 => {
            // OPTIONS - handled via options field below
        }
        2 => {
            emit_keyword(e, "CONNECTION");
            e.space();
            emit_single_quoted_str(e, &n.conninfo);
        }
        3 => {
            e.token(TokenKind::SET_KW);
            e.space();
            emit_keyword(e, "PUBLICATION");
            e.space();
            emit_comma_separated_list(e, &n.publication, super::emit_node);
        }
        4 => {
            emit_keyword(e, "ADD");
            e.space();
            emit_keyword(e, "PUBLICATION");
            e.space();
            emit_comma_separated_list(e, &n.publication, super::emit_node);
        }
        5 => {
            e.token(TokenKind::DROP_KW);
            e.space();
            emit_keyword(e, "PUBLICATION");
            e.space();
            emit_comma_separated_list(e, &n.publication, super::emit_node);
        }
        6 => {
            emit_keyword(e, "REFRESH");
            e.space();
            emit_keyword(e, "PUBLICATION");
        }
        7 => {
            emit_keyword(e, "ENABLE");
        }
        8 => {
            emit_keyword(e, "SKIP");
        }
        _ => {}
    }

    // Options
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
