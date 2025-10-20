use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::{NodeEnum, protobuf::CreateSubscriptionStmt};

use super::string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str};

pub(super) fn emit_create_subscription_stmt(e: &mut EventEmitter, n: &CreateSubscriptionStmt) {
    e.group_start(GroupKind::CreateSubscriptionStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    emit_keyword(e, "SUBSCRIPTION");
    e.space();
    emit_identifier_maybe_quoted(e, &n.subname);

    e.space();
    emit_keyword(e, "CONNECTION");
    e.space();
    // Emit connection string as string literal
    emit_single_quoted_str(e, &n.conninfo);

    e.space();
    emit_keyword(e, "PUBLICATION");
    e.space();
    emit_comma_separated_list(e, &n.publication, |node, e| {
        if let Some(NodeEnum::String(s)) = &node.node {
            emit_identifier_maybe_quoted(e, &s.sval);
        }
    });

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
