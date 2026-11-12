//! Questions about the shape of a node that decide how it is printed.

use pgls_query::protobuf::{AExpr, AExprKind, BoolExprType, SubLinkType, a_const::Val};
use pgls_query::{Node, NodeEnum};

use crate::emitter::EventEmitter;

pub(crate) fn is_window_expression(node: &Node) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::FuncCall(call)) => call.over.is_some(),
        Some(NodeEnum::AExpr(expr)) => {
            expr.lexpr.as_deref().is_some_and(is_window_expression)
                || expr.rexpr.as_deref().is_some_and(is_window_expression)
        }
        _ => false,
    }
}

/// Whether a select item without an alias, which would continue on a line of its own, is written as
/// a name or keyword followed by
/// parentheses that hold everything that can break: a function call, `coalesce(...)`, `cast(...)`,
/// a subquery, ...
pub(crate) fn breaks_only_inside_its_parentheses(e: &EventEmitter, item: &Node) -> bool {
    let Some(NodeEnum::ResTarget(target)) = item.node.as_ref() else {
        return false;
    };
    if !target.name.is_empty() || !target.indirection.is_empty() {
        return false;
    }
    let Some(value) = target.val.as_deref() else {
        return false;
    };
    match value.node.as_ref() {
        // FILTER, WITHIN GROUP and OVER continue the call on lines of their own.
        Some(NodeEnum::FuncCall(call)) => {
            call.agg_filter.is_none() && call.over.is_none() && !call.agg_within_group
        }
        Some(NodeEnum::CoalesceExpr(_) | NodeEnum::MinMaxExpr(_) | NodeEnum::XmlSerialize(_)) => {
            true
        }
        Some(NodeEnum::TypeCast(_)) => matches!(e.config().cast_style, crate::CastStyle::Cast),
        Some(NodeEnum::SubLink(_)) => breaks_only_inside_a_subquery(value),
        _ => false,
    }
}

pub(crate) fn breaks_only_inside_a_subquery(node: &Node) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::SubLink(sub_link)) => {
            sub_link.sub_link_type != SubLinkType::MultiexprSublink as i32
        }
        Some(NodeEnum::AExpr(expr)) => {
            expr.kind() == AExprKind::AexprOp
                && expr.lexpr.as_deref().is_some_and(is_simple_operand)
                && expr.rexpr.as_deref().is_some_and(|rexpr| {
                    matches!(
                        rexpr.node.as_ref(),
                        Some(NodeEnum::SubLink(sub_link))
                            if sub_link.sub_link_type == SubLinkType::ExprSublink as i32
                                || sub_link.sub_link_type == SubLinkType::ArraySublink as i32
                    )
                })
        }
        Some(NodeEnum::BoolExpr(expr)) => {
            expr.boolop() == BoolExprType::NotExpr
                && matches!(expr.args.as_slice(), [arg] if breaks_only_inside_a_subquery(arg))
        }
        _ => false,
    }
}

/// A column reference, parameter or constant, which never breaks.
fn is_simple_operand(node: &Node) -> bool {
    matches!(
        node.node.as_ref(),
        Some(NodeEnum::ColumnRef(_) | NodeEnum::ParamRef(_) | NodeEnum::AConst(_))
    )
}

/// Where a node sits, which decides whether it needs parentheses.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Position {
    /// An operand of an operator, `parent`, on `side`.
    Operand {
        parent: Option<OperatorInfo>,
        side: OperandSide,
    },
    /// An operand of AND, OR or NOT.
    BoolOperand(BoolExprType),
    /// The operand of `::type` or of other high-precedence syntax such as `AT TIME ZONE`.
    Postfix,
}

/// Whether `node` needs parentheses at `position` to keep its meaning, or, for nested AND and OR,
/// to read clearly.
pub(crate) fn needs_parentheses(node: &Node, position: Position) -> bool {
    match position {
        Position::Operand { parent, side } => operand_needs_parentheses(node, parent, side),
        Position::BoolOperand(parent_kind) => bool_operand_needs_parentheses(node, parent_kind),
        Position::Postfix => postfix_operand_needs_parentheses(node),
    }
}

pub(crate) const PREC_UNARY: u8 = 90;
pub(crate) const PREC_POWER: u8 = 80;
pub(crate) const PREC_MULTIPLICATIVE: u8 = 70;
pub(crate) const PREC_ADDITIVE: u8 = 60;
pub(crate) const PREC_OTHER: u8 = 55;
pub(crate) const PREC_COMPARISON: u8 = 45;
pub(crate) const PREC_IS: u8 = 40;
pub(crate) const PREC_NOT: u8 = 35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OperatorInfo {
    pub(crate) precedence: u8,
    pub(crate) associativity: OperatorAssociativity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OperatorAssociativity {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OperatorArity {
    Unary,
    Binary,
}

impl OperatorArity {
    pub(crate) fn from_aexpr(expr: &AExpr) -> Self {
        match (expr.lexpr.as_ref(), expr.rexpr.as_ref()) {
            (Some(_), Some(_)) => OperatorArity::Binary,
            (None, Some(_)) | (Some(_), None) => OperatorArity::Unary,
            (None, None) => OperatorArity::Unary,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OperandSide {
    Left,
    Right,
    Unary,
}

pub(crate) fn node_operator_info(node: &Node) -> Option<OperatorInfo> {
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

pub(crate) fn operator_info(name: &[Node], arity: OperatorArity) -> Option<OperatorInfo> {
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

pub(crate) fn operator_symbol(name: &[Node]) -> Option<&str> {
    name.last().and_then(|node| match node.node.as_ref()? {
        NodeEnum::String(s) => Some(s.sval.as_str()),
        _ => None,
    })
}

fn operand_needs_parentheses(node: &Node, parent: Option<OperatorInfo>, side: OperandSide) -> bool {
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

/// Whether an operand is wrapped in parentheses: a weaker operator always needs them, and an AND
/// inside an OR keeps them for readability, although AND already binds tighter.
fn bool_operand_needs_parentheses(node: &Node, parent_kind: BoolExprType) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::BoolExpr(child)) => {
            bool_precedence(child.boolop()) < bool_precedence(parent_kind)
                || (parent_kind == BoolExprType::OrExpr && child.boolop() == BoolExprType::AndExpr)
        }
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

/// Whitelist of self contained expressions that can carry a `::`, or be the operand of other
/// high-precedence syntax such as `AT TIME ZONE`, without parentheses.
///
/// Anything absent from this list is parenthesised, which is always valid SQL. Listing the unsafe
/// kinds instead would fail open on any node nobody thought about.
fn postfix_operand_needs_parentheses(node: &Node) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::AExpr(a_expr)) => a_expr.kind != AExprKind::AexprNullif as i32,
        // A negative constant is emitted with its sign, and `-1::int` casts 1 before negating it.
        Some(NodeEnum::AConst(a_const)) => match a_const.val.as_ref() {
            Some(Val::Ival(i)) => i.ival < 0,
            Some(Val::Fval(f)) => f.fval.starts_with('-'),
            _ => false,
        },
        Some(
            NodeEnum::ColumnRef(_)
            | NodeEnum::ParamRef(_)
            | NodeEnum::FuncCall(_)
            | NodeEnum::NullIfExpr(_)
            | NodeEnum::TypeCast(_)
            | NodeEnum::SubLink(_)
            | NodeEnum::CaseExpr(_)
            | NodeEnum::CoalesceExpr(_)
            | NodeEnum::MinMaxExpr(_)
            | NodeEnum::ArrayExpr(_)
            | NodeEnum::RowExpr(_)
            | NodeEnum::AIndirection(_)
            | NodeEnum::AArrayExpr(_),
        ) => false,
        _ => true,
    }
}
