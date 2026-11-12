use pgls_query::protobuf::ResTarget;

use super::prelude::*;

pub(super) fn emit_res_target(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start();

    if let Some(ref val) = n.val {
        emit_node(e, val);

        if !n.name.is_empty() {
            if let Some(location) = val
                .node
                .as_ref()
                .and_then(|node| crate::comments::last_node_location(node.to_ref()))
            {
                emit_comments_before_keyword_after(e, "as", location);
            }

            let case_alias_in_expanded_layout =
                matches!(val.node.as_ref(), Some(NodeEnum::CaseExpr(_)))
                    && matches!(e.config().layout, crate::Layout::Expanded);

            if is_window_expression(val) {
                e.group_start();
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AS_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.name);
                e.group_end();
            } else {
                if case_alias_in_expanded_layout {
                    e.space();
                } else {
                    e.line(LineType::SoftOrSpace);
                }
                e.token(TokenKind::AS_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.name);
            }
        }
    }

    e.group_end();
}
