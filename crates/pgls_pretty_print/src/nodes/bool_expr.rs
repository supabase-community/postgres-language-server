use pgls_query::protobuf::{BoolExpr, BoolExprType};
use pgls_query::{Node, NodeEnum};

use crate::{
    LogicalOperatorPlacement, TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_bool_expr(e: &mut EventEmitter, n: &BoolExpr) {
    e.group_start(GroupKind::BoolExpr);

    match n.boolop() {
        BoolExprType::AndExpr => emit_variadic_bool_expr(e, n, TokenKind::AND_KW),
        BoolExprType::OrExpr => emit_variadic_bool_expr(e, n, TokenKind::OR_KW),
        BoolExprType::NotExpr => emit_not_expr(e, n),
        BoolExprType::Undefined => unreachable!("Undefined BoolExprType"),
    }

    e.group_end();
}

fn emit_variadic_bool_expr(e: &mut EventEmitter, n: &BoolExpr, keyword: TokenKind) {
    let parent_prec = bool_precedence(n.boolop());
    let leading = matches!(
        e.config().logical_operator_placement,
        LogicalOperatorPlacement::Leading
    );

    for (idx, arg) in n.args.iter().enumerate() {
        if idx > 0 {
            if leading {
                // The break opportunity sits before the keyword, so a broken condition reads
                // "\n\tAND b = 2" while a single line one still reads "a = 1 AND b = 2".
                if matches!(e.config().layout, crate::Layout::Expanded) {
                    super::emit_layout_break(e);
                } else {
                    e.line(LineType::SoftOrSpace);
                }
                e.token(keyword.clone());
                e.space();
            } else {
                e.space();
                e.token(keyword.clone());
                if matches!(e.config().layout, crate::Layout::Expanded) {
                    super::emit_layout_break(e);
                } else {
                    e.line(LineType::SoftOrSpace);
                }
            }
        }

        emit_bool_operand(e, arg, parent_prec);
    }
}

fn emit_not_expr(e: &mut EventEmitter, n: &BoolExpr) {
    e.token(TokenKind::NOT_KW);

    if n.args.len() != 1 {
        panic!(
            "NOT expressions should have exactly one argument, got {}",
            n.args.len()
        );
    }

    if let Some(arg) = n.args.first() {
        e.space();
        emit_bool_operand(e, arg, bool_precedence(BoolExprType::NotExpr));
    }
}

fn emit_bool_operand(e: &mut EventEmitter, node: &Node, parent_prec: u8) {
    e.with_leading_comment_line_break(|e| {
        if needs_parentheses(node, parent_prec) {
            e.token(TokenKind::L_PAREN);
            super::emit_node(node, e);
            e.token(TokenKind::R_PAREN);
        } else {
            super::emit_node(node, e);
        }
    });
}

fn needs_parentheses(node: &Node, parent_prec: u8) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::BoolExpr(child)) => bool_precedence(child.boolop()) < parent_prec,
        _ => false,
    }
}

fn bool_precedence(kind: BoolExprType) -> u8 {
    match kind {
        BoolExprType::NotExpr => 3,
        BoolExprType::AndExpr => 2,
        BoolExprType::OrExpr => 1,
        BoolExprType::Undefined => 0,
    }
}
