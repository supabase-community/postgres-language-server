use pgls_query::protobuf::{BoolExpr, BoolExprType};

use crate::LogicalOperatorPlacement;

use super::prelude::*;

pub(super) fn emit_bool_expr(e: &mut EventEmitter, n: &BoolExpr) {
    emit_bool_expr_as(e, n, OperandGroup::Own);
}

/// Which group the operands of an AND or OR chain break with.
#[derive(Clone, Copy)]
pub(super) enum OperandGroup {
    /// A group of the chain's own.
    Own,
    /// The enclosing group, so that the operands go on their own lines whenever it breaks. A NOT
    /// keeps a group of its own.
    Enclosing,
}

pub(super) fn emit_bool_expr_as(e: &mut EventEmitter, n: &BoolExpr, group: OperandGroup) {
    match (group, n.boolop()) {
        (OperandGroup::Enclosing, BoolExprType::AndExpr) => {
            emit_variadic_bool_expr(e, n, TokenKind::AND_KW)
        }
        (OperandGroup::Enclosing, BoolExprType::OrExpr) => {
            emit_variadic_bool_expr(e, n, TokenKind::OR_KW)
        }
        _ => {
            e.group_start();

            match n.boolop() {
                BoolExprType::AndExpr => emit_variadic_bool_expr(e, n, TokenKind::AND_KW),
                BoolExprType::OrExpr => emit_variadic_bool_expr(e, n, TokenKind::OR_KW),
                BoolExprType::NotExpr => emit_not_expr(e, n),
                BoolExprType::Undefined => unreachable!("Undefined BoolExprType"),
            }

            e.expression_group_end();
        }
    }
}

fn emit_variadic_bool_expr(e: &mut EventEmitter, n: &BoolExpr, keyword: TokenKind) {
    let mut is_first = true;
    emit_variadic_operands(e, n, &keyword, &mut is_first);
}

/// Emits the operands of an AND or OR chain. A nested chain of the same operator is emitted as part
/// of this one: `(a AND b) AND c` reads as `a AND b AND c`, which it means, and lays out the same
/// way it does once formatted.
fn emit_variadic_operands(
    e: &mut EventEmitter,
    n: &BoolExpr,
    keyword: &TokenKind,
    is_first: &mut bool,
) {
    let parent_kind = n.boolop();
    let leading = matches!(
        e.config().logical_operator_placement,
        LogicalOperatorPlacement::Leading
    );

    for arg in &n.args {
        if let Some(NodeEnum::BoolExpr(nested)) = arg.node.as_ref()
            && nested.boolop() == parent_kind
        {
            emit_with_comments_at(e, nested.location, |e| {
                emit_variadic_operands(e, nested, keyword, is_first);
            });
            continue;
        }

        if !*is_first {
            if leading {
                if let Some(location) = arg
                    .node
                    .as_ref()
                    .and_then(crate::comments::first_node_location)
                {
                    e.take_own_line_leading_comments_at(location);
                }
                // The break opportunity sits before the keyword, so a broken condition reads
                // "\n\tAND b = 2" while a single line one still reads "a = 1 AND b = 2".
                if matches!(e.config().layout, crate::Layout::Expanded) {
                    emit_layout_break(e);
                } else {
                    e.line(LineType::SoftOrSpace);
                }
                e.token(keyword.clone());
                e.space();
            } else {
                e.space();
                e.token(keyword.clone());
                if matches!(e.config().layout, crate::Layout::Expanded) {
                    emit_layout_break(e);
                } else {
                    e.line(LineType::SoftOrSpace);
                }
            }
        }
        *is_first = false;

        emit_bool_operand(e, arg, parent_kind);
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
        emit_bool_operand(e, arg, BoolExprType::NotExpr);
    }
}

fn emit_bool_operand(e: &mut EventEmitter, node: &Node, parent_kind: BoolExprType) {
    e.with_leading_comment_line_break(|e| {
        if needs_parentheses(node, Position::BoolOperand(parent_kind)) {
            // The parentheses get their own group so that `or (a and b)` stays on one line when it
            // fits, even after the enclosing chain broke. Once it breaks, the operands go on their
            // own lines, indented, and the closing parenthesis returns to the operand's indent.
            e.group_start();
            e.token(TokenKind::L_PAREN);
            e.indent_start();
            e.line(LineType::Soft);
            emit_node(e, node);
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();
        } else {
            emit_node(e, node);
        }
    });
}
