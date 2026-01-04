use pgls_query::{
    Node,
    protobuf::{LimitOption, SelectStmt, SetOperation},
};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::{
    node_list::emit_comma_separated_list, string::emit_keyword, window_def::emit_window_definition,
};

pub(super) fn emit_select_stmt(e: &mut EventEmitter, n: &SelectStmt) {
    emit_select_stmt_impl(e, n, true);
}

pub(super) fn emit_select_stmt_no_semicolon(e: &mut EventEmitter, n: &SelectStmt) {
    emit_select_stmt_impl(e, n, false);
}

fn emit_select_stmt_impl(e: &mut EventEmitter, n: &SelectStmt, with_semicolon: bool) {
    e.group_start(GroupKind::SelectStmt);

    // Emit WITH clause (Common Table Expressions) if present
    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    // Check if this is a set operation (UNION/INTERSECT/EXCEPT)
    match n.op() {
        SetOperation::SetopUnion | SetOperation::SetopIntersect | SetOperation::SetopExcept => {
            // Emit left operand - wrap in parentheses if it's a set operation that needs precedence protection
            // or if it has ORDER BY/LIMIT/OFFSET that would be ambiguous
            if let Some(ref larg) = n.larg {
                let needs_parens_left = needs_set_operation_parens(n.op(), larg.op(), true)
                    || select_needs_parens(larg);
                if needs_parens_left {
                    e.token(TokenKind::L_PAREN);
                }
                emit_select_stmt_no_semicolon(e, larg);
                if needs_parens_left {
                    e.token(TokenKind::R_PAREN);
                }
            }

            // Emit set operation keyword
            e.line(LineType::SoftOrSpace);
            match n.op() {
                SetOperation::SetopUnion => e.token(TokenKind::UNION_KW),
                SetOperation::SetopIntersect => e.token(TokenKind::INTERSECT_KW),
                SetOperation::SetopExcept => e.token(TokenKind::EXCEPT_KW),
                _ => unreachable!(),
            }

            // Emit ALL keyword if present
            if n.all {
                e.space();
                e.token(TokenKind::ALL_KW);
            }

            // Emit right operand - wrap in parentheses if it's a set operation that needs precedence protection
            // or if it has ORDER BY/LIMIT/OFFSET that would be ambiguous
            e.line(LineType::SoftOrSpace);
            if let Some(ref rarg) = n.rarg {
                let needs_parens_right = needs_set_operation_parens(n.op(), rarg.op(), false)
                    || select_needs_parens(rarg);
                if needs_parens_right {
                    e.token(TokenKind::L_PAREN);
                }
                emit_select_stmt_no_semicolon(e, rarg);
                if needs_parens_right {
                    e.token(TokenKind::R_PAREN);
                }
            }

            // Emit ORDER BY clause if present on the set operation
            if !n.sort_clause.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::ORDER_KW);
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                e.indent_start();
                emit_comma_separated_list(e, &n.sort_clause, super::emit_node);
                e.indent_end();
            }

            // Emit LIMIT/OFFSET clauses
            match n.limit_option() {
                LimitOption::WithTies => {
                    if let Some(ref offset) = n.limit_offset {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::OFFSET_KW);
                        e.space();
                        super::emit_node(offset, e);
                        e.space();
                        e.token(TokenKind::ROWS_KW);
                    }

                    if let Some(ref limit) = n.limit_count {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::FETCH_KW);
                        e.space();
                        e.token(TokenKind::FIRST_KW);
                        e.space();
                        super::emit_node(limit, e);
                        e.space();
                        e.token(TokenKind::ROWS_KW);
                        e.space();
                        e.token(TokenKind::WITH_KW);
                        e.space();
                        emit_keyword(e, "TIES");
                    }
                }
                _ => {
                    if let Some(ref offset) = n.limit_offset {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::OFFSET_KW);
                        e.space();
                        super::emit_node(offset, e);
                    }

                    if let Some(ref limit) = n.limit_count {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::LIMIT_KW);
                        e.space();
                        super::emit_node(limit, e);
                    }
                }
            }

            // Emit locking clause if present
            for locking in &n.locking_clause {
                e.line(LineType::SoftOrSpace);
                super::emit_node(locking, e);
            }

            if with_semicolon {
                e.token(TokenKind::SEMICOLON);
            }

            e.group_end();
            return;
        }
        SetOperation::SetopNone | SetOperation::Undefined => {
            // Not a set operation, continue with regular SELECT
        }
    }

    // Check if this is a VALUES clause (used in INSERT statements)
    if !n.values_lists.is_empty() {
        e.token(TokenKind::VALUES_KW);

        // VALUES should be compact when short: VALUES (1), (2), (3)
        // Only break to multiple lines when needed
        e.line(LineType::SoftOrSpace);
        e.indent_start();

        for (i, row) in n.values_lists.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
            }
            // Wrap each row tuple in a group so it can break independently
            e.group_start(GroupKind::List);
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Soft);
            e.indent_start();
            super::emit_node(row, e);
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();
        }

        e.indent_end();

        if with_semicolon {
            e.token(TokenKind::SEMICOLON);
        }
    } else {
        e.token(TokenKind::SELECT_KW);

        if !n.distinct_clause.is_empty() {
            emit_distinct_clause(e, &n.distinct_clause);
        }

        if !n.target_list.is_empty() {
            e.indent_start();
            e.line(LineType::SoftOrSpace);

            emit_comma_separated_list(e, &n.target_list, super::emit_node);

            e.indent_end();
        }

        // Emit INTO clause if present (SELECT ... INTO table_name)
        if let Some(ref into_clause) = n.into_clause {
            e.line(LineType::SoftOrSpace);
            super::emit_into_clause(e, into_clause);
        }

        if !n.from_clause.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::FROM_KW);
            e.line(LineType::SoftOrSpace);

            e.indent_start();

            emit_comma_separated_list(e, &n.from_clause, super::emit_node);

            e.indent_end();
        }

        if let Some(ref where_clause) = n.where_clause {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WHERE_KW);
            super::emit_clause_condition(e, where_clause);
        }

        // Emit GROUP BY clause if present
        if !n.group_clause.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::GROUP_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            // Emit DISTINCT if group_distinct is set
            if n.group_distinct {
                e.space();
                e.token(TokenKind::DISTINCT_KW);
            }
            e.space();
            e.indent_start();
            emit_comma_separated_list(e, &n.group_clause, super::emit_node);
            e.indent_end();
        }

        // Emit HAVING clause if present
        if let Some(ref having_clause) = n.having_clause {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::HAVING_KW);
            super::emit_clause_condition(e, having_clause);
        }

        // Emit WINDOW clause if present
        if !n.window_clause.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WINDOW_KW);
            e.line(LineType::SoftOrSpace);
            e.indent_start();
            for (idx, window) in n.window_clause.iter().enumerate() {
                if idx > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }

                if let Some(pgls_query::NodeEnum::WindowDef(window_def)) = window.node.as_ref() {
                    emit_window_definition(e, window_def);
                } else {
                    super::emit_node(window, e);
                }
            }
            e.indent_end();
        }

        // Emit ORDER BY clause if present
        if !n.sort_clause.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            e.indent_start();
            emit_comma_separated_list(e, &n.sort_clause, super::emit_node);
            e.indent_end();
        }

        match n.limit_option() {
            LimitOption::WithTies => {
                if let Some(ref limit_offset) = n.limit_offset {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::OFFSET_KW);
                    e.space();
                    super::emit_node(limit_offset, e);
                    e.space();
                    e.token(TokenKind::ROWS_KW);
                }

                if let Some(ref limit_count) = n.limit_count {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::FETCH_KW);
                    e.space();
                    e.token(TokenKind::FIRST_KW);
                    e.space();
                    super::emit_node(limit_count, e);
                    e.space();
                    e.token(TokenKind::ROWS_KW);
                    e.space();
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    emit_keyword(e, "TIES");
                }
            }
            _ => {
                if let Some(ref limit_count) = n.limit_count {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::LIMIT_KW);
                    e.space();
                    super::emit_node(limit_count, e);
                }

                if let Some(ref limit_offset) = n.limit_offset {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::OFFSET_KW);
                    e.space();
                    super::emit_node(limit_offset, e);
                }
            }
        }

        if !n.locking_clause.is_empty() {
            for locking in &n.locking_clause {
                if let Some(pgls_query::NodeEnum::LockingClause(locking_clause)) =
                    locking.node.as_ref()
                {
                    e.line(LineType::SoftOrSpace);
                    super::emit_locking_clause(e, locking_clause);
                }
            }
        }

        if with_semicolon {
            e.token(TokenKind::SEMICOLON);
        }
    }

    e.group_end();
}

fn emit_distinct_clause(e: &mut EventEmitter, clause: &[Node]) {
    e.space();
    e.token(TokenKind::DISTINCT_KW);

    let distinct_exprs: Vec<&Node> = clause.iter().filter(|node| node.node.is_some()).collect();

    if distinct_exprs.is_empty() {
        return;
    }

    e.space();
    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);

    for (idx, node) in distinct_exprs.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }
        super::emit_node(node, e);
    }

    e.indent_end();
    e.token(TokenKind::R_PAREN);
}

/// Determines if we need parentheses around a set operation operand.
/// Check if a SelectStmt needs parentheses when used as a child of a set operation.
/// This is needed when the SELECT has ORDER BY, LIMIT, or OFFSET clauses that
/// would be ambiguous without parentheses, or when it has a WITH clause.
fn select_needs_parens(stmt: &SelectStmt) -> bool {
    // SELECTs (including set operations) with ORDER BY/LIMIT/OFFSET/WITH need parens
    // when used as a child of a set operation
    !stmt.sort_clause.is_empty()
        || stmt.limit_count.is_some()
        || stmt.limit_offset.is_some()
        || stmt.with_clause.is_some()
}

/// SQL set operations have this precedence: INTERSECT > UNION = EXCEPT
/// UNION and EXCEPT are left-associative.
///
/// We need parentheses when:
/// 1. Right operand has the same operator (to preserve right-to-left grouping if user wrote it that way)
/// 2. Child operator has lower precedence than parent
/// 3. Child has UNION/EXCEPT and parent has INTERSECT (different precedence)
fn needs_set_operation_parens(
    parent_op: SetOperation,
    child_op: SetOperation,
    is_left_operand: bool,
) -> bool {
    // No parentheses needed if child is not a set operation
    match child_op {
        SetOperation::SetopNone | SetOperation::Undefined => return false,
        _ => {}
    }

    // INTERSECT has higher precedence than UNION/EXCEPT
    // If parent is INTERSECT and child is UNION/EXCEPT, no parens needed (child binds tighter naturally)
    // If parent is UNION/EXCEPT and child is INTERSECT, no parens needed (INTERSECT binds tighter)

    // Parentheses are needed:
    // 1. When child is UNION/EXCEPT and parent is INTERSECT (to preserve order)
    //    -> Actually no, INTERSECT binds tighter, so this doesn't need parens
    // 2. When parent is UNION and child is EXCEPT on the right (or vice versa) - same precedence, need to preserve structure
    // 3. When parent and child are the same operator and it's right operand (right-associative grouping)

    // Same operator on right side needs parentheses to preserve structure
    // E.g., A UNION (B UNION C) needs parens, otherwise it's (A UNION B) UNION C
    if !is_left_operand && child_op == parent_op {
        return true;
    }

    // Different operators at same precedence level need parens on right to preserve structure
    // UNION and EXCEPT have same precedence
    match (parent_op, child_op) {
        // UNION and EXCEPT have same precedence, so right-side needs parens
        (SetOperation::SetopUnion, SetOperation::SetopExcept)
        | (SetOperation::SetopExcept, SetOperation::SetopUnion) => !is_left_operand,
        // INTERSECT has higher precedence - if child is INTERSECT, no parens needed
        // If parent is INTERSECT and child is UNION/EXCEPT, we need parens
        (SetOperation::SetopIntersect, SetOperation::SetopUnion)
        | (SetOperation::SetopIntersect, SetOperation::SetopExcept) => {
            // Parent INTERSECT binds tighter than child UNION/EXCEPT
            // So child needs parens to be evaluated first
            true
        }
        // If parent is UNION/EXCEPT and child is INTERSECT, no parens needed
        // INTERSECT already binds tighter
        (SetOperation::SetopUnion, SetOperation::SetopIntersect)
        | (SetOperation::SetopExcept, SetOperation::SetopIntersect) => false,
        _ => false,
    }
}
