use pgt_query::protobuf::{BoolExpr, BoolExprType};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_keyword_separated_list,
};

pub(super) fn emit_bool_expr(e: &mut EventEmitter, n: &BoolExpr) {
    e.group_start(GroupKind::BoolExpr);

    match n.boolop() {
        BoolExprType::AndExpr => emit_keyword_separated_list(e, &n.args, TokenKind::AND_KW),
        BoolExprType::OrExpr => emit_keyword_separated_list(e, &n.args, TokenKind::OR_KW),
        BoolExprType::NotExpr => {
            e.token(crate::TokenKind::NOT_KW);
            e.space();
            assert!(
                n.args.len() == 1,
                "NOT expressions should have exactly one argument"
            );
            let arg = &n.args[0];
            super::emit_node(arg, e);
        }
        BoolExprType::Undefined => unreachable!("Undefined BoolExprType"),
    }

    e.group_end();
}
