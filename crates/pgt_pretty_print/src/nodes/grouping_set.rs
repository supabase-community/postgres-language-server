use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{GroupingSet, GroupingSetKind};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_grouping_set(e: &mut EventEmitter, n: &GroupingSet) {
    e.group_start(GroupKind::GroupingSet);

    match n.kind {
        kind if kind == GroupingSetKind::GroupingSetRollup as i32 => {
            e.token(TokenKind::IDENT("ROLLUP".to_string()));
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.content, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        kind if kind == GroupingSetKind::GroupingSetCube as i32 => {
            e.token(TokenKind::IDENT("CUBE".to_string()));
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.content, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        kind if kind == GroupingSetKind::GroupingSetSets as i32 => {
            e.token(TokenKind::IDENT("GROUPING".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SETS".to_string()));
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.content, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        kind if kind == GroupingSetKind::GroupingSetSimple as i32 => {
            // Simple grouping set: just emit the content without wrapper
            emit_comma_separated_list(e, &n.content, super::emit_node);
        }
        kind if kind == GroupingSetKind::GroupingSetEmpty as i32 => {
            // Empty grouping set: ()
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Default: emit as simple list
            emit_comma_separated_list(e, &n.content, super::emit_node);
        }
    }

    e.group_end();
}
