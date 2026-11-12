use pgls_query::protobuf::{AExpr, AExprKind};

use super::prelude::*;

pub(super) fn emit_a_expr(e: &mut EventEmitter, n: &AExpr) {
    if let Some(location) = n
        .lexpr
        .as_ref()
        .and_then(|node| node.node.as_ref())
        .and_then(|node| crate::comments::first_node_location(node.to_ref()))
    {
        e.take_leading_comments_at(location);
    }
    e.group_start();

    match n.kind() {
        AExprKind::AexprOp => emit_aexpr_op(e, n),
        AExprKind::AexprOpAny => emit_aexpr_op_any(e, n),
        AExprKind::AexprOpAll => emit_aexpr_op_all(e, n),
        AExprKind::AexprDistinct => emit_aexpr_distinct(e, n),
        AExprKind::AexprNotDistinct => emit_aexpr_not_distinct(e, n),
        AExprKind::AexprNullif => emit_aexpr_nullif(e, n),
        AExprKind::AexprIn => emit_aexpr_in(e, n),
        AExprKind::AexprLike => emit_aexpr_like(e, n),
        AExprKind::AexprIlike => emit_aexpr_ilike(e, n),
        AExprKind::AexprSimilar => emit_aexpr_similar(e, n),
        AExprKind::AexprBetween => emit_aexpr_between(e, n),
        AExprKind::AexprNotBetween => emit_aexpr_not_between(e, n),
        AExprKind::AexprBetweenSym => emit_aexpr_between_sym(e, n),
        AExprKind::AexprNotBetweenSym => emit_aexpr_not_between_sym(e, n),
        AExprKind::Undefined => {}
    }

    e.expression_group_end();
}

// Basic binary operator: left op right
fn emit_aexpr_op(e: &mut EventEmitter, n: &AExpr) {
    if n.name.is_empty() {
        if let Some(ref lexpr) = n.lexpr {
            emit_node(e, lexpr);
        }
        if let Some(ref rexpr) = n.rexpr {
            if n.lexpr.is_some() {
                e.space();
            }
            emit_node(e, rexpr);
        }
        return;
    }

    let parent_info = operator_info(&n.name, OperatorArity::from_aexpr(n));

    match (n.lexpr.as_ref(), n.rexpr.as_ref()) {
        (Some(lexpr), Some(rexpr)) => {
            emit_operand_with_parens(e, lexpr, parent_info, OperandSide::Left);

            if operator_prefers_line_break(&n.name) {
                // Keep the operator attached to the left-hand side and allow the
                // right-hand side to wrap underneath when the expression exceeds
                // the line width.
                let grouped_suffix = is_window_expression(lexpr) || is_window_expression(rexpr);
                if grouped_suffix {
                    e.group_start();
                }
                e.space();
                emit_operator(e, &n.name);
                // A subquery breaks inside its parentheses, so `= (` stays on the operator line,
                // like `in (` and `exists (`.
                if starts_with_subquery_parenthesis(rexpr) {
                    e.space();
                } else {
                    e.line(LineType::SoftOrSpace);
                }
                emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Right);
                if grouped_suffix {
                    e.group_end();
                }
            } else {
                e.space();
                emit_operator(e, &n.name);
                e.space();
                emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Right);
            }
        }
        (None, Some(rexpr)) => {
            if let Some(op) = extract_simple_operator(&n.name) {
                if op.eq_ignore_ascii_case("not") {
                    e.token(TokenKind::NOT_KW);
                    if operator_prefers_line_break(&n.name) {
                        e.line(LineType::SoftOrSpace);
                    } else {
                        e.space();
                    }
                    emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Unary);
                } else {
                    emit_simple_operator(e, op);
                    if operator_needs_space(op) {
                        if operator_prefers_line_break(&n.name) {
                            e.line(LineType::SoftOrSpace);
                        } else {
                            e.space();
                        }
                    } else if !needs_parentheses(
                        rexpr,
                        Position::Operand {
                            parent: parent_info,
                            side: OperandSide::Unary,
                        },
                    ) && starts_with_operator_char(rexpr)
                    {
                        // Without a space the two operators would lex as one token, or as the
                        // start of a comment when both are minus signs.
                        e.space();
                    }
                    emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Unary);
                }
            } else {
                emit_operator(e, &n.name);
                if operator_prefers_line_break(&n.name) {
                    e.line(LineType::SoftOrSpace);
                } else {
                    e.space();
                }
                emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Unary);
            }
        }
        (Some(lexpr), None) => {
            emit_operand_with_parens(e, lexpr, parent_info, OperandSide::Left);
            if let Some(op) = extract_simple_operator(&n.name) {
                if operator_needs_space(op) {
                    if operator_prefers_line_break(&n.name) {
                        e.line(LineType::SoftOrSpace);
                    } else {
                        e.space();
                    }
                }
                emit_simple_operator(e, op);
            } else {
                if operator_prefers_line_break(&n.name) {
                    e.line(LineType::SoftOrSpace);
                } else {
                    e.space();
                }
                emit_operator(e, &n.name);
            }
        }
        (None, None) => {}
    }
}

// expr op ANY (subquery)
fn emit_aexpr_op_any(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.space();
    }

    if !n.name.is_empty() {
        if let Some(op) = extract_simple_operator(&n.name) {
            emit_simple_operator(e, op);
        } else {
            emit_operator(e, &n.name);
        }
        e.space();
    }

    e.token(TokenKind::ANY_KW);
    e.space();
    e.token(TokenKind::L_PAREN);

    if let Some(ref rexpr) = n.rexpr {
        emit_node(e, rexpr);
    }

    e.token(TokenKind::R_PAREN);
}

// expr op ALL (subquery)
fn emit_aexpr_op_all(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.space();
    }

    if !n.name.is_empty() {
        if let Some(op) = extract_simple_operator(&n.name) {
            emit_simple_operator(e, op);
        } else {
            emit_operator(e, &n.name);
        }
        e.space();
    }

    e.token(TokenKind::ALL_KW);
    e.space();
    e.token(TokenKind::L_PAREN);

    if let Some(ref rexpr) = n.rexpr {
        emit_node(e, rexpr);
    }

    e.token(TokenKind::R_PAREN);
}

// expr IS DISTINCT FROM expr2
fn emit_aexpr_distinct(e: &mut EventEmitter, n: &AExpr) {
    // IS DISTINCT FROM has PREC_IS (40) precedence
    let parent_info = Some(OperatorInfo {
        precedence: PREC_IS,
        associativity: OperatorAssociativity::Left,
    });

    if let Some(ref lexpr) = n.lexpr {
        emit_operand_with_parens(e, lexpr, parent_info, OperandSide::Left);
        e.space();
    }

    e.token(TokenKind::IS_KW);
    e.space();
    e.token(TokenKind::DISTINCT_KW);
    e.space();
    e.token(TokenKind::FROM_KW);
    // Like a comparison operator, the right-hand side wraps when the expression doesn't fit.
    e.line(LineType::SoftOrSpace);

    if let Some(ref rexpr) = n.rexpr {
        emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Right);
    }
}

// expr IS NOT DISTINCT FROM expr2
fn emit_aexpr_not_distinct(e: &mut EventEmitter, n: &AExpr) {
    // IS NOT DISTINCT FROM has PREC_IS (40) precedence
    let parent_info = Some(OperatorInfo {
        precedence: PREC_IS,
        associativity: OperatorAssociativity::Left,
    });

    if let Some(ref lexpr) = n.lexpr {
        emit_operand_with_parens(e, lexpr, parent_info, OperandSide::Left);
        e.space();
    }

    e.token(TokenKind::IS_KW);
    e.space();
    e.token(TokenKind::NOT_KW);
    e.space();
    e.token(TokenKind::DISTINCT_KW);
    e.space();
    e.token(TokenKind::FROM_KW);
    // Like a comparison operator, the right-hand side wraps when the expression doesn't fit.
    e.line(LineType::SoftOrSpace);

    if let Some(ref rexpr) = n.rexpr {
        emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Right);
    }
}

// NULLIF(expr, expr2)
fn emit_aexpr_nullif(e: &mut EventEmitter, n: &AExpr) {
    e.token(TokenKind::NULLIF_KW);
    e.token(TokenKind::L_PAREN);

    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
    }

    e.token(TokenKind::COMMA);
    e.space();

    if let Some(ref rexpr) = n.rexpr {
        emit_node(e, rexpr);
    }

    e.token(TokenKind::R_PAREN);
}

// expr IN (values)
fn emit_aexpr_in(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.line(LineType::SoftOrSpace);
    }

    let is_not = extract_simple_operator(&n.name)
        .map(|op| op == "<>")
        .unwrap_or(false);

    if is_not {
        e.token(TokenKind::NOT_KW);
        e.space();
    }

    e.token(TokenKind::IN_KW);
    e.space();

    // The rexpr is typically a List node, which emits comma-separated items
    // We need to wrap it in parentheses for IN clause
    if let Some(ref rexpr) = n.rexpr {
        match rexpr.node.as_ref() {
            Some(NodeEnum::SubLink(_)) => emit_node(e, rexpr),
            Some(NodeEnum::List(list)) => {
                e.token(TokenKind::L_PAREN);
                e.indent_start();
                e.line(LineType::Soft);
                e.group_start();
                emit_fill_comma_separated_list(e, &list.items, emit_node);
                e.group_end();
                e.indent_end();
                e.line(LineType::Soft);
                e.token(TokenKind::R_PAREN);
            }
            _ => {
                e.token(TokenKind::L_PAREN);
                e.indent_start();
                e.line(LineType::Soft);
                emit_node(e, rexpr);
                e.indent_end();
                e.line(LineType::Soft);
                e.token(TokenKind::R_PAREN);
            }
        }
    }
}

// expr LIKE pattern [ESCAPE escape]
fn emit_aexpr_like(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.space();
    }

    let is_not = extract_simple_operator(&n.name)
        .map(|op| op == "!~~")
        .unwrap_or(false);

    if is_not {
        e.token(TokenKind::NOT_KW);
        e.space();
    }

    e.token(TokenKind::LIKE_KW);
    // Like a comparison operator, the pattern wraps when the expression doesn't fit.
    e.line(LineType::SoftOrSpace);

    if let Some(ref rexpr) = n.rexpr {
        emit_node(e, rexpr);
    }
}

// expr ILIKE pattern [ESCAPE escape]
fn emit_aexpr_ilike(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.space();
    }

    let is_not = extract_simple_operator(&n.name)
        .map(|op| op == "!~~*")
        .unwrap_or(false);

    if is_not {
        e.token(TokenKind::NOT_KW);
        e.space();
    }

    e.token(TokenKind::ILIKE_KW);
    // Like a comparison operator, the pattern wraps when the expression doesn't fit.
    e.line(LineType::SoftOrSpace);

    if let Some(ref rexpr) = n.rexpr {
        emit_node(e, rexpr);
    }
}

// expr SIMILAR TO pattern [ESCAPE escape]
fn emit_aexpr_similar(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.space();
    }

    let is_not = extract_simple_operator(&n.name)
        .map(|op| op == "!~")
        .unwrap_or(false);

    if is_not {
        e.token(TokenKind::NOT_KW);
        e.space();
    }

    e.token(TokenKind::SIMILAR_KW);
    e.space();
    e.token(TokenKind::TO_KW);
    // Like a comparison operator, the pattern wraps when the expression doesn't fit.
    e.line(LineType::SoftOrSpace);

    // PostgreSQL stores SIMILAR TO as a call to similar_to_escape(pattern) or similar_to_escape(pattern, escape)
    // We need to emit the original pattern (and ESCAPE clause if present)
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::FuncCall(func)) = rexpr.node.as_ref()
            && func.funcname.iter().any(|n| {
                matches!(n.node.as_ref(), Some(pgls_query::NodeEnum::String(s)) if s.sval == "similar_to_escape")
            }) {
                // Emit just the pattern (first argument)
                if !func.args.is_empty() {
                    emit_node(e, &func.args[0]);
                }
                // If there's a second argument, it's the ESCAPE character
                if func.args.len() >= 2 {
                    e.space();
                    e.token(TokenKind::ESCAPE_KW);
                    e.space();
                    emit_node(e, &func.args[1]);
                }
                return;
            }
        // Fallback for other patterns
        emit_node(e, rexpr);
    }
}

// expr BETWEEN expr2 AND expr3
fn emit_aexpr_between(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::BETWEEN_KW);
    e.space();

    // rexpr is a List node with two elements, but we need "expr AND expr" not "expr, expr"
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::List(list)) = rexpr.node.as_ref() {
            if !list.items.is_empty() {
                emit_node(e, &list.items[0]);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                emit_node(e, &list.items[1]);
            }
        } else {
            emit_node(e, rexpr);
        }
    }
}

// expr NOT BETWEEN expr2 AND expr3
fn emit_aexpr_not_between(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::NOT_KW);
    e.space();
    e.token(TokenKind::BETWEEN_KW);
    e.space();

    // rexpr is a List node with two elements, but we need "expr AND expr" not "expr, expr"
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::List(list)) = rexpr.node.as_ref() {
            if !list.items.is_empty() {
                emit_node(e, &list.items[0]);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                emit_node(e, &list.items[1]);
            }
        } else {
            emit_node(e, rexpr);
        }
    }
}

// expr BETWEEN SYMMETRIC expr2 AND expr3
fn emit_aexpr_between_sym(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::BETWEEN_KW);
    e.space();
    e.token(TokenKind::SYMMETRIC_KW);
    e.space();

    // rexpr is a List node with two elements, but we need "expr AND expr" not "expr, expr"
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::List(list)) = rexpr.node.as_ref() {
            if !list.items.is_empty() {
                emit_node(e, &list.items[0]);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                emit_node(e, &list.items[1]);
            }
        } else {
            emit_node(e, rexpr);
        }
    }
}

// expr NOT BETWEEN SYMMETRIC expr2 AND expr3
fn emit_aexpr_not_between_sym(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        emit_node(e, lexpr);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::NOT_KW);
    e.space();
    e.token(TokenKind::BETWEEN_KW);
    e.space();
    e.token(TokenKind::SYMMETRIC_KW);
    e.space();

    // rexpr is a List node with two elements, but we need "expr AND expr" not "expr, expr"
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::List(list)) = rexpr.node.as_ref() {
            if !list.items.is_empty() {
                emit_node(e, &list.items[0]);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                emit_node(e, &list.items[1]);
            }
        } else {
            emit_node(e, rexpr);
        }
    }
}

fn emit_simple_operator(e: &mut EventEmitter, op: &str) {
    e.token(TokenKind::IDENT(op.to_string()));
}

fn extract_simple_operator(name: &[Node]) -> Option<&str> {
    if name.len() != 1 {
        return None;
    }

    match name[0].node.as_ref() {
        Some(NodeEnum::String(s)) => Some(&s.sval),
        _ => None,
    }
}

/// Whether the emitted operand begins with an operator character: a prefix operator, or a numeric
/// constant carrying its own minus sign.
fn starts_with_operator_char(node: &Node) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::AExpr(expr)) => {
            matches!(expr.kind(), AExprKind::AexprOp)
                && expr.lexpr.is_none()
                && extract_simple_operator(&expr.name).is_some_and(|op| !operator_needs_space(op))
        }
        Some(NodeEnum::AConst(c)) => match c.val.as_ref() {
            Some(pgls_query::protobuf::a_const::Val::Ival(i)) => i.ival < 0,
            Some(pgls_query::protobuf::a_const::Val::Fval(f)) => f.fval.starts_with('-'),
            _ => false,
        },
        _ => false,
    }
}

fn operator_needs_space(op: &str) -> bool {
    op.chars().any(|c| c.is_alphabetic())
}

/// Whether `node` is printed as `(subquery)` or `array(subquery)`.
fn starts_with_subquery_parenthesis(node: &Node) -> bool {
    use pgls_query::protobuf::SubLinkType;

    matches!(
        node.node.as_ref(),
        Some(NodeEnum::SubLink(sub_link))
            if sub_link.sub_link_type == SubLinkType::ExprSublink as i32
                || sub_link.sub_link_type == SubLinkType::ArraySublink as i32
    )
}

fn operator_prefers_line_break(name: &[Node]) -> bool {
    match extract_simple_operator(name) {
        // Comparison operators
        Some("=") | Some("<>") | Some("!=") | Some("<") | Some(">") | Some("<=") | Some(">=")
        // Logical operators
        | Some("||") | Some("AND") | Some("and") | Some("OR") | Some("or")
        // Arithmetic operators - allow breaking around these for narrow widths
        | Some("+") | Some("-") | Some("*") | Some("/") | Some("%") | Some("^")
        // Bitwise operators
        | Some("&") | Some("|") | Some("#") | Some("~") | Some("<<") | Some(">>")
        // Other common operators
        | Some("@") | Some("@@") | Some("->") | Some("->>") | Some("#>") | Some("#>>")
        | Some("?") | Some("?|") | Some("?&") | Some("@>") | Some("<@") | Some("&&") => true,
        // For any other operator, still allow line breaks - they're likely user-defined
        // and we want consistent formatting behavior
        _ => true,
    }
}

fn emit_operand_with_parens(
    e: &mut EventEmitter,
    node: &Node,
    parent: Option<OperatorInfo>,
    side: OperandSide,
) {
    if needs_parentheses(node, Position::Operand { parent, side }) {
        e.token(TokenKind::L_PAREN);
        emit_node(e, node);
        e.token(TokenKind::R_PAREN);
    } else {
        emit_node(e, node);
    }
}
