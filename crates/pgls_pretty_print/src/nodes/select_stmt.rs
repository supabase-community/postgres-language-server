use pgls_query::protobuf::{LimitOption, SelectStmt, SetOperation};

use crate::{ClauseBodyStyle, Comment, Layout};

use super::prelude::*;

pub(super) fn emit_select_stmt(e: &mut EventEmitter, n: &SelectStmt) {
    e.group_start();

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
                let needs_parens_left =
                    needs_set_operation_parens(n.op(), larg.op(), OperandSide::Left)
                        || select_needs_parens(larg);
                if needs_parens_left {
                    e.token(TokenKind::L_PAREN);
                }
                emit_select_stmt(e, larg);
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
                let needs_parens_right =
                    needs_set_operation_parens(n.op(), rarg.op(), OperandSide::Right)
                        || select_needs_parens(rarg);
                if needs_parens_right {
                    e.token(TokenKind::L_PAREN);
                }
                emit_select_stmt(e, rarg);
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
                emit_comma_separated_list(e, &n.sort_clause, emit_node);
                e.indent_end();
            }

            // Emit LIMIT/OFFSET clauses
            match n.limit_option() {
                LimitOption::WithTies => {
                    if let Some(ref offset) = n.limit_offset {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::OFFSET_KW);
                        e.space();
                        emit_node(e, offset);
                        e.space();
                        e.token(TokenKind::ROWS_KW);
                    }

                    if let Some(ref limit) = n.limit_count {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::FETCH_KW);
                        e.space();
                        e.token(TokenKind::FIRST_KW);
                        e.space();
                        emit_node(e, limit);
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
                        emit_node(e, offset);
                    }

                    if let Some(ref limit) = n.limit_count {
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::LIMIT_KW);
                        e.space();
                        emit_node(e, limit);
                    }
                }
            }

            // Emit locking clause if present
            for locking in &n.locking_clause {
                e.line(LineType::SoftOrSpace);
                emit_node(e, locking);
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

        e.indent_start();
        let spacing = match e.config().values_row_layout {
            Layout::Fit => {
                e.line(LineType::SoftOrSpace);
                ListSeparatorSpacing::SoftOrSpace
            }
            Layout::Expanded => {
                e.line(LineType::Hard);
                ListSeparatorSpacing::Hard
            }
        };

        let last_row = n.values_lists.last();
        emit_comma_separated_list_with_spacing(e, &n.values_lists, spacing, |e, row| {
            // A line comment written after a row is attached to the row's last value. Emitted
            // there, it would end up inside the parentheses and force the row to break, so it
            // follows the closing parenthesis instead, and the comma when there is one.
            let followed_by_comma = !last_row.is_some_and(|last| std::ptr::eq(last, row));
            let comments_after_row = take_comments_after_row(e, row, followed_by_comma);

            // Wrap each row tuple in a group so it can break independently
            e.group_start();
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Soft);
            e.indent_start();
            emit_node(e, row);
            e.indent_end();
            e.line(LineType::Soft);
            e.token(TokenKind::R_PAREN);
            e.group_end();

            e.emit_trailing_comments(comments_after_row);
        });

        e.indent_end();
    } else {
        e.token(TokenKind::SELECT_KW);

        if !n.distinct_clause.is_empty() {
            emit_distinct_clause(e, &n.distinct_clause);
        }

        if !n.target_list.is_empty() {
            // Compact keeps a lone item on the SELECT line, `select 1`; a list of several still
            // puts one item per line. The indent is for the item's continuation lines, which an
            // item that only breaks inside its parentheses, `coalesce(...)`, doesn't have: indented,
            // its arguments would sit two levels in and its `)` one level in.
            let lone_compact_item =
                matches!(e.config().clause_body_style, ClauseBodyStyle::Compact)
                    && n.target_list.len() == 1;
            let indent =
                !(lone_compact_item && breaks_only_inside_its_parentheses(e, &n.target_list[0]));

            if indent {
                e.indent_start();
            }
            if lone_compact_item {
                e.space();
            } else {
                emit_layout_break(e);
            }

            emit_comma_separated_list_with_layout_break(e, &n.target_list, emit_node);

            if indent {
                e.indent_end();
            }
        }

        // Emit INTO clause if present (SELECT ... INTO table_name)
        if let Some(ref into_clause) = n.into_clause {
            emit_layout_break(e);
            super::emit_into_clause(e, into_clause);
        }

        if !n.from_clause.is_empty() {
            emit_layout_break(e);
            e.token(TokenKind::FROM_KW);

            // Compact keeps the first relation on the FROM line; the joins that follow still
            // break onto their own indented lines.
            match e.config().clause_body_style {
                ClauseBodyStyle::Compact => e.space(),
                ClauseBodyStyle::Break => e.line(LineType::SoftOrSpace),
            }

            e.indent_start();

            emit_comma_separated_list(e, &n.from_clause, emit_node);

            e.indent_end();
        }

        if let Some(ref where_clause) = n.where_clause {
            emit_layout_break(e);
            e.token(TokenKind::WHERE_KW);
            emit_clause_condition(e, where_clause);
        }

        // Emit GROUP BY clause if present
        if !n.group_clause.is_empty() {
            emit_layout_break(e);
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
            emit_comma_separated_list(e, &n.group_clause, emit_node);
            e.indent_end();
        }

        // Emit HAVING clause if present
        if let Some(ref having_clause) = n.having_clause {
            emit_layout_break(e);
            e.token(TokenKind::HAVING_KW);
            emit_clause_condition(e, having_clause);
        }

        // Emit WINDOW clause if present
        if !n.window_clause.is_empty() {
            emit_layout_break(e);
            e.token(TokenKind::WINDOW_KW);
            match e.config().clause_body_style {
                ClauseBodyStyle::Compact => e.space(),
                ClauseBodyStyle::Break => e.line(LineType::SoftOrSpace),
            }
            e.indent_start();
            for (idx, window) in n.window_clause.iter().enumerate() {
                if idx > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }

                if let Some(pgls_query::NodeEnum::WindowDef(window_def)) = window.node.as_ref() {
                    super::emit_window_def_as(e, window_def, super::WindowForm::Named);
                } else {
                    emit_node(e, window);
                }
            }
            e.indent_end();
        }
    }

    // ORDER BY, LIMIT and locking follow a select list and a VALUES list alike.
    if !n.sort_clause.is_empty() {
        emit_layout_break(e);
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        e.indent_start();
        emit_comma_separated_list(e, &n.sort_clause, emit_node);
        e.indent_end();
    }

    match n.limit_option() {
        LimitOption::WithTies => {
            if let Some(ref limit_offset) = n.limit_offset {
                emit_layout_break(e);
                e.token(TokenKind::OFFSET_KW);
                e.space();
                emit_node(e, limit_offset);
                e.space();
                e.token(TokenKind::ROWS_KW);
            }

            if let Some(ref limit_count) = n.limit_count {
                emit_layout_break(e);
                e.token(TokenKind::FETCH_KW);
                e.space();
                e.token(TokenKind::FIRST_KW);
                e.space();
                emit_node(e, limit_count);
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
                emit_layout_break(e);
                e.token(TokenKind::LIMIT_KW);
                e.space();
                emit_node(e, limit_count);
            }

            if let Some(ref limit_offset) = n.limit_offset {
                emit_layout_break(e);
                e.token(TokenKind::OFFSET_KW);
                e.space();
                emit_node(e, limit_offset);
            }
        }
    }

    if !n.locking_clause.is_empty() {
        for locking in &n.locking_clause {
            if let Some(pgls_query::NodeEnum::LockingClause(locking_clause)) = locking.node.as_ref()
            {
                emit_layout_break(e);
                super::emit_locking_clause(e, locking_clause);
            }
        }
    }

    e.group_end();
}

/// Removes the line comments written after `row`, which are attached to its last value, so that
/// they can be emitted after the row. Between two rows, they follow the comma.
fn take_comments_after_row(
    e: &mut EventEmitter,
    row: &Node,
    followed_by_comma: bool,
) -> Vec<Comment> {
    let Some(location) = row
        .node
        .as_ref()
        .and_then(crate::comments::last_node_location)
    else {
        return Vec::new();
    };

    let (mut after_row, inside): (Vec<_>, Vec<_>) = e
        .remove_trailing_comments_at(location)
        .into_iter()
        .partition(|comment| comment.line_comment && !comment.own_line);
    e.restore_trailing_comments_at(location, inside);

    if followed_by_comma {
        for comment in &mut after_row {
            comment.after_separator = true;
        }
    }
    after_row
}

fn emit_distinct_clause(e: &mut EventEmitter, clause: &[Node]) {
    e.space();
    e.token(TokenKind::DISTINCT_KW);

    let distinct_exprs: Vec<&Node> = clause.iter().filter(|node| node.node.is_some()).collect();

    if distinct_exprs.is_empty() {
        return;
    }

    // The clause has its own group so that it can stay on one line while the target list breaks,
    // which is the usual shape of a DISTINCT ON query.
    e.group_start();

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
        emit_node(e, node);
    }

    e.indent_end();
    // Symmetric with the Soft after the opening parenthesis: without it the closing parenthesis
    // stays glued to the last expression when the list breaks.
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);

    e.group_end();
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
    side: OperandSide,
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
    if matches!(side, OperandSide::Right) && child_op == parent_op {
        return true;
    }

    // Different operators at same precedence level need parens on right to preserve structure
    // UNION and EXCEPT have same precedence
    match (parent_op, child_op) {
        // UNION and EXCEPT have same precedence, so right-side needs parens
        (SetOperation::SetopUnion, SetOperation::SetopExcept)
        | (SetOperation::SetopExcept, SetOperation::SetopUnion) => {
            matches!(side, OperandSide::Right)
        }
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
