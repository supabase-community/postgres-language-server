use pgls_query::protobuf::Alias;

use super::prelude::*;

pub(super) fn emit_alias(e: &mut EventEmitter, n: &Alias) {
    e.group_start();

    if n.aliasname.is_empty() {
        e.group_end();
        return;
    }

    e.token(TokenKind::AS_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.aliasname);

    // Add column aliases if present
    if !n.colnames.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_parenthesized_list(e, &n.colnames, |e, node| {
            // Column names in alias are String nodes
            if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                emit_identifier_maybe_quoted(e, &s.sval);
            } else {
                emit_node(e, node);
            }
        });
    }

    e.group_end();
}
