use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::{emit_comma_separated_list, emit_parenthesized_list},
};
use pgls_query::{NodeEnum, protobuf::CreateSubscriptionStmt};

use super::string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str};

pub(super) fn emit_create_subscription_stmt(e: &mut EventEmitter, n: &CreateSubscriptionStmt) {
    e.group_start(GroupKind::CreateSubscriptionStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    emit_keyword(e, "SUBSCRIPTION");
    e.space();
    emit_identifier_maybe_quoted(e, &n.subname);

    // The clauses continue the statement, so a broken one indents them.
    e.indent_start();
    e.line(LineType::SoftOrSpace);
    emit_keyword(e, "CONNECTION");
    e.space();
    // Emit connection string as string literal
    emit_single_quoted_str(e, &n.conninfo);

    e.line(LineType::SoftOrSpace);
    emit_keyword(e, "PUBLICATION");
    e.space();
    e.group_start(GroupKind::List);
    emit_comma_separated_list(e, &n.publication, |node, e| {
        if let Some(NodeEnum::String(s)) = &node.node {
            emit_identifier_maybe_quoted(e, &s.sval);
        }
    });
    e.group_end();

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, super::emit_node);
    }
    e.indent_end();

    e.group_end();
}
