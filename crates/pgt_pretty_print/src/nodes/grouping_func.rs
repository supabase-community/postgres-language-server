use pgt_query::protobuf::GroupingFunc;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_grouping_func(e: &mut EventEmitter, n: &GroupingFunc) {
    e.group_start(GroupKind::GroupingFunc);

    e.token(TokenKind::IDENT("GROUPING".to_string()));
    e.token(TokenKind::L_PAREN);

    if !n.args.is_empty() {
        super::node_list::emit_comma_separated_list(e, &n.args, super::emit_node);
    }

    e.token(TokenKind::R_PAREN);
    e.group_end();
}
