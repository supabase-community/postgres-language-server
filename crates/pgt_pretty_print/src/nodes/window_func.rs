use pgt_query::protobuf::WindowFunc;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

/// Emit a WindowFunc (planner window function node)
/// These are internal planner representations with function OIDs
/// We emit a simple fallback representation
pub(super) fn emit_window_func(e: &mut EventEmitter, n: &WindowFunc) {
    e.group_start(GroupKind::WindowFunc);

    // WindowFunc is the planner's representation of window functions
    // Without access to pg_proc, we emit a placeholder with the OID
    if n.winfnoid != 0 {
        e.token(TokenKind::IDENT(format!("winfunc#{}", n.winfnoid)));
    } else {
        e.token(TokenKind::IDENT("window_func".to_string()));
    }

    e.token(TokenKind::L_PAREN);

    if n.winstar {
        e.token(TokenKind::IDENT("*".to_string()));
    } else if !n.args.is_empty() {
        emit_comma_separated_list(e, &n.args, super::emit_node);
    }

    e.token(TokenKind::R_PAREN);

    if let Some(ref filter) = n.aggfilter {
        e.space();
        e.token(TokenKind::FILTER_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::WHERE_KW);
        e.space();
        super::emit_node(filter, e);
        e.token(TokenKind::R_PAREN);
    }

    e.space();
    e.token(TokenKind::OVER_KW);
    e.space();
    e.token(TokenKind::L_PAREN);
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
