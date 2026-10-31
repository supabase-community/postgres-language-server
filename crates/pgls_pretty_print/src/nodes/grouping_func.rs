use pgls_query::protobuf::GroupingFunc;

use super::prelude::*;

pub(super) fn emit_grouping_func(e: &mut EventEmitter, n: &GroupingFunc) {
    e.group_start(GroupKind::GroupingFunc);

    e.token(TokenKind::GROUPING_KW);
    e.token(TokenKind::L_PAREN);

    if !n.args.is_empty() {
        emit_comma_separated_list(e, &n.args, emit_node);
    }

    e.token(TokenKind::R_PAREN);
    e.group_end();
}
