use pgt_query::protobuf::{Aggref, Node};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};

/// Emit an Aggref (planner aggregate function node)
/// These are internal planner representations with function OIDs
/// We emit a simple fallback representation
pub(super) fn emit_aggref(e: &mut EventEmitter, n: &Aggref) {
    e.group_start(GroupKind::Aggref);

    // Aggref is the planner's representation of aggregate functions
    // Without access to pg_proc, we emit a placeholder with the OID
    if n.aggfnoid != 0 {
        e.token(TokenKind::IDENT(format!("agg#{}", n.aggfnoid)));
    } else {
        e.token(TokenKind::IDENT("agg".to_string()));
    }

    e.token(TokenKind::L_PAREN);

    if n.aggstar {
        e.token(TokenKind::IDENT("*".to_string()));
    } else {
        let mut emitted_any = false;

        if !n.aggdistinct.is_empty() && !n.args.is_empty() {
            e.token(TokenKind::DISTINCT_KW);
            e.space();
        }

        emitted_any = emit_node_sequence(e, &n.aggdirectargs, emitted_any);
        emit_node_sequence(e, &n.args, emitted_any);
    }

    e.token(TokenKind::R_PAREN);

    if !n.aggorder.is_empty() {
        e.space();
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.aggorder, super::emit_node);
    }

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

    e.group_end();
}

fn emit_node_sequence(e: &mut EventEmitter, nodes: &[Node], mut emitted_any: bool) -> bool {
    for node in nodes {
        if emitted_any {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }
        super::emit_node(node, e);
        emitted_any = true;
    }

    emitted_any
}
