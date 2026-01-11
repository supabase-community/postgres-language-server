use pgls_query::protobuf::{AExpr, AExprKind};
use pgls_query::{Node, NodeEnum};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_a_expr(e: &mut EventEmitter, n: &AExpr) {
    e.group_start(GroupKind::AExpr);

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

    e.group_end();
}

// Basic binary operator: left op right
fn emit_aexpr_op(e: &mut EventEmitter, n: &AExpr) {
    if n.name.is_empty() {
        if let Some(ref lexpr) = n.lexpr {
            super::emit_node(lexpr, e);
        }
        if let Some(ref rexpr) = n.rexpr {
            if n.lexpr.is_some() {
                e.space();
            }
            super::emit_node(rexpr, e);
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
                e.space();
                emit_operator(e, &n.name);
                e.line(LineType::SoftOrSpace);
                emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Right);
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
        super::emit_node(lexpr, e);
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
        super::emit_node(rexpr, e);
    }

    e.token(TokenKind::R_PAREN);
}

// expr op ALL (subquery)
fn emit_aexpr_op_all(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
        super::emit_node(rexpr, e);
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
    e.space();

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
    e.space();

    if let Some(ref rexpr) = n.rexpr {
        emit_operand_with_parens(e, rexpr, parent_info, OperandSide::Right);
    }
}

// NULLIF(expr, expr2)
fn emit_aexpr_nullif(e: &mut EventEmitter, n: &AExpr) {
    e.token(TokenKind::NULLIF_KW);
    e.token(TokenKind::L_PAREN);

    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
    }

    e.token(TokenKind::COMMA);
    e.space();

    if let Some(ref rexpr) = n.rexpr {
        super::emit_node(rexpr, e);
    }

    e.token(TokenKind::R_PAREN);
}

// expr IN (values)
fn emit_aexpr_in(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
            Some(NodeEnum::SubLink(_)) => super::emit_node(rexpr, e),
            _ => {
                e.token(TokenKind::L_PAREN);
                e.indent_start();
                e.line(LineType::Soft);
                super::emit_node(rexpr, e);
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
        super::emit_node(lexpr, e);
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
    e.space();

    if let Some(ref rexpr) = n.rexpr {
        super::emit_node(rexpr, e);
    }
}

// expr ILIKE pattern [ESCAPE escape]
fn emit_aexpr_ilike(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
    e.space();

    if let Some(ref rexpr) = n.rexpr {
        super::emit_node(rexpr, e);
    }
}

// expr SIMILAR TO pattern [ESCAPE escape]
fn emit_aexpr_similar(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
    e.space();

    // PostgreSQL stores SIMILAR TO as a call to similar_to_escape(pattern) or similar_to_escape(pattern, escape)
    // We need to emit the original pattern (and ESCAPE clause if present)
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::FuncCall(func)) = rexpr.node.as_ref() {
            if func.funcname.iter().any(|n| {
                matches!(n.node.as_ref(), Some(pgls_query::NodeEnum::String(s)) if s.sval == "similar_to_escape")
            }) {
                // Emit just the pattern (first argument)
                if !func.args.is_empty() {
                    super::emit_node(&func.args[0], e);
                }
                // If there's a second argument, it's the ESCAPE character
                if func.args.len() >= 2 {
                    e.space();
                    e.token(TokenKind::ESCAPE_KW);
                    e.space();
                    super::emit_node(&func.args[1], e);
                }
                return;
            }
        }
        // Fallback for other patterns
        super::emit_node(rexpr, e);
    }
}

// expr BETWEEN expr2 AND expr3
fn emit_aexpr_between(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::BETWEEN_KW);
    e.space();

    // rexpr is a List node with two elements, but we need "expr AND expr" not "expr, expr"
    if let Some(ref rexpr) = n.rexpr {
        if let Some(pgls_query::NodeEnum::List(list)) = rexpr.node.as_ref() {
            if !list.items.is_empty() {
                super::emit_node(&list.items[0], e);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                super::emit_node(&list.items[1], e);
            }
        } else {
            super::emit_node(rexpr, e);
        }
    }
}

// expr NOT BETWEEN expr2 AND expr3
fn emit_aexpr_not_between(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
                super::emit_node(&list.items[0], e);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                super::emit_node(&list.items[1], e);
            }
        } else {
            super::emit_node(rexpr, e);
        }
    }
}

// expr BETWEEN SYMMETRIC expr2 AND expr3
fn emit_aexpr_between_sym(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
                super::emit_node(&list.items[0], e);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                super::emit_node(&list.items[1], e);
            }
        } else {
            super::emit_node(rexpr, e);
        }
    }
}

// expr NOT BETWEEN SYMMETRIC expr2 AND expr3
fn emit_aexpr_not_between_sym(e: &mut EventEmitter, n: &AExpr) {
    if let Some(ref lexpr) = n.lexpr {
        super::emit_node(lexpr, e);
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
                super::emit_node(&list.items[0], e);
            }
            if list.items.len() >= 2 {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::AND_KW);
                e.space();
                super::emit_node(&list.items[1], e);
            }
        } else {
            super::emit_node(rexpr, e);
        }
    }
}

fn emit_operator(e: &mut EventEmitter, name: &[Node]) {
    if name.len() > 1 {
        emit_qualified_operator(e, name);
    } else if let Some(first) = name.first() {
        emit_operator_part(e, first);
    }
}

fn emit_qualified_operator(e: &mut EventEmitter, name: &[Node]) {
    e.token(TokenKind::OPERATOR_KW);
    e.token(TokenKind::L_PAREN);

    for (idx, part) in name.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }
        emit_operator_part(e, part);
    }

    e.token(TokenKind::R_PAREN);
}

fn emit_operator_part(e: &mut EventEmitter, node: &Node) {
    match node.node.as_ref() {
        Some(NodeEnum::String(s)) => e.token(TokenKind::IDENT(s.sval.clone())),
        _ => super::emit_node(node, e),
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

fn operator_needs_space(op: &str) -> bool {
    op.chars().any(|c| c.is_alphabetic())
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

const PREC_UNARY: u8 = 90;
const PREC_POWER: u8 = 80;
const PREC_MULTIPLICATIVE: u8 = 70;
const PREC_ADDITIVE: u8 = 60;
const PREC_OTHER: u8 = 55;
const PREC_COMPARISON: u8 = 45;
const PREC_IS: u8 = 40;
const PREC_NOT: u8 = 35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OperatorInfo {
    precedence: u8,
    associativity: OperatorAssociativity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorAssociativity {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorArity {
    Unary,
    Binary,
}

impl OperatorArity {
    fn from_aexpr(expr: &AExpr) -> Self {
        match (expr.lexpr.as_ref(), expr.rexpr.as_ref()) {
            (Some(_), Some(_)) => OperatorArity::Binary,
            (None, Some(_)) | (Some(_), None) => OperatorArity::Unary,
            (None, None) => OperatorArity::Unary,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperandSide {
    Left,
    Right,
    Unary,
}

fn emit_operand_with_parens(
    e: &mut EventEmitter,
    node: &Node,
    parent: Option<OperatorInfo>,
    side: OperandSide,
) {
    if needs_parentheses(node, parent, side) {
        e.token(TokenKind::L_PAREN);
        super::emit_node(node, e);
        e.token(TokenKind::R_PAREN);
    } else {
        super::emit_node(node, e);
    }
}

fn needs_parentheses(node: &Node, parent: Option<OperatorInfo>, side: OperandSide) -> bool {
    let Some(parent_info) = parent else {
        return false;
    };

    let Some(child_info) = node_operator_info(node) else {
        return false;
    };

    if matches!(side, OperandSide::Unary) {
        return child_info.precedence < parent_info.precedence;
    }

    if child_info.precedence < parent_info.precedence {
        return true;
    }

    if child_info.precedence > parent_info.precedence {
        return false;
    }

    // Same precedence - PostgreSQL comparison operators are non-associative
    // (you can't chain a < b < c), so always need parens when nesting
    // comparison expressions at the same level
    if parent_info.precedence == PREC_COMPARISON && child_info.precedence == PREC_COMPARISON {
        return true;
    }

    match parent_info.associativity {
        OperatorAssociativity::Left => matches!(side, OperandSide::Right),
        OperatorAssociativity::Right => matches!(side, OperandSide::Left),
    }
}

fn node_operator_info(node: &Node) -> Option<OperatorInfo> {
    match node.node.as_ref()? {
        NodeEnum::AExpr(expr) if matches!(expr.kind(), AExprKind::AexprOp) => {
            operator_info(&expr.name, OperatorArity::from_aexpr(expr))
        }
        // IS NULL / IS NOT NULL have PREC_IS precedence
        NodeEnum::NullTest(_) => Some(OperatorInfo {
            precedence: PREC_IS,
            associativity: OperatorAssociativity::Left,
        }),
        // Boolean expressions (AND/OR) need parens when nested in comparison
        // Use a low precedence so they get wrapped
        NodeEnum::BoolExpr(_) => Some(OperatorInfo {
            precedence: 20, // Lower than PREC_NOT (35)
            associativity: OperatorAssociativity::Left,
        }),
        _ => None,
    }
}

fn operator_info(name: &[Node], arity: OperatorArity) -> Option<OperatorInfo> {
    let symbol = operator_symbol(name)?.to_ascii_lowercase();

    let (precedence, associativity) = match symbol.as_str() {
        "+" | "-" if matches!(arity, OperatorArity::Unary) => {
            (PREC_UNARY, OperatorAssociativity::Right)
        }
        "~" if matches!(arity, OperatorArity::Unary) => (PREC_UNARY, OperatorAssociativity::Right),
        "!" if matches!(arity, OperatorArity::Unary) => (PREC_UNARY, OperatorAssociativity::Left),
        "^" => (PREC_POWER, OperatorAssociativity::Left),
        "*" | "/" | "%" => (PREC_MULTIPLICATIVE, OperatorAssociativity::Left),
        "+" | "-" => (PREC_ADDITIVE, OperatorAssociativity::Left),
        "||" | "<<" | ">>" | "#" | "&" | "|" => (PREC_OTHER, OperatorAssociativity::Left),
        "not" => (PREC_NOT, OperatorAssociativity::Right),
        "=" | "<" | ">" | "<=" | ">=" | "<>" | "!=" => {
            (PREC_COMPARISON, OperatorAssociativity::Left)
        }
        "is" | "isnull" | "notnull" => (PREC_IS, OperatorAssociativity::Left),
        _ => (PREC_OTHER, OperatorAssociativity::Left),
    };

    Some(OperatorInfo {
        precedence,
        associativity,
    })
}

fn operator_symbol(name: &[Node]) -> Option<&str> {
    name.last().and_then(|node| match node.node.as_ref()? {
        NodeEnum::String(s) => Some(s.sval.as_str()),
        _ => None,
    })
}
