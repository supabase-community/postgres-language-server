use pgls_query::protobuf::{JoinExpr, JoinType};

use super::prelude::*;

pub(super) fn emit_join_expr(e: &mut EventEmitter, n: &JoinExpr) {
    e.group_start();

    // If the join has an alias, we need to wrap it in parentheses
    let has_alias = n.alias.is_some();
    if has_alias {
        e.token(TokenKind::L_PAREN);
    }

    // Left side
    if let Some(ref larg) = n.larg {
        emit_node(e, larg);
    }

    if n.larg.is_some() {
        // A join may start with any of these, and is written with the ones its type needs.
        if let Some(ref rarg) = n.rarg {
            for keyword in ["natural", "inner", "left", "right", "full", "cross", "join"] {
                emit_comments_before_keyword(e, keyword, rarg);
            }
        }

        if matches!(e.config().layout, crate::Layout::Expanded) {
            // A hard break makes every soft line in the current group break too. Close the left
            // operand's group before emitting it so the joined table and qualification can still
            // fit.
            e.group_end();
            emit_layout_break(e);
            e.group_start();
        } else {
            e.line(LineType::SoftOrSpace);
        }
    }

    let mut first_token = true;
    let mut emit_join_token = |token: TokenKind, e: &mut EventEmitter| {
        if !first_token {
            e.space();
        }
        e.token(token);
        first_token = false;
    };

    if n.is_natural {
        emit_join_token(TokenKind::NATURAL_KW, e);
    }

    let jointype = n.jointype();

    match jointype {
        JoinType::JoinInner => {
            if !n.is_natural {
                emit_join_token(TokenKind::INNER_KW, e);
            }
        }
        JoinType::JoinLeft => {
            emit_join_token(TokenKind::LEFT_KW, e);
            if !n.is_natural {
                emit_join_token(TokenKind::OUTER_KW, e);
            }
        }
        JoinType::JoinRight => {
            emit_join_token(TokenKind::RIGHT_KW, e);
            if !n.is_natural {
                emit_join_token(TokenKind::OUTER_KW, e);
            }
        }
        JoinType::JoinFull => {
            emit_join_token(TokenKind::FULL_KW, e);
            if !n.is_natural {
                emit_join_token(TokenKind::OUTER_KW, e);
            }
        }
        JoinType::JoinSemi => {
            emit_join_token(TokenKind::IDENT("semi".to_string()), e);
        }
        JoinType::JoinAnti => {
            emit_join_token(TokenKind::IDENT("anti".to_string()), e);
        }
        JoinType::JoinRightAnti => {
            emit_join_token(TokenKind::RIGHT_KW, e);
            emit_join_token(TokenKind::IDENT("anti".to_string()), e);
        }
        JoinType::JoinUniqueOuter | JoinType::JoinUniqueInner | JoinType::Undefined => {
            emit_join_token(TokenKind::CROSS_KW, e);
        }
    }

    emit_join_token(TokenKind::JOIN_KW, e);

    // Right side - allow break after JOIN keyword for narrow widths
    if let Some(ref rarg) = n.rarg {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_node(e, rarg);
        e.indent_end();
    }

    // Join qualification
    if !n.using_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_parenthesized_list(e, &n.using_clause, |e, node| {
            // For USING clause, String nodes should be identifiers
            if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                emit_identifier_maybe_quoted(e, &s.sval);
            } else {
                emit_node(e, node);
            }
        });

        // Emit USING alias if present (PostgreSQL 14+)
        // Format: USING (col1, col2) AS alias
        if let Some(ref using_alias) = n.join_using_alias {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            emit_identifier_maybe_quoted(e, &using_alias.aliasname);
        }
    } else if let Some(ref quals) = n.quals {
        emit_comments_before_keyword(e, "on", quals);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        emit_condition_on_keyword_line(e, quals);
    } else if matches!(jointype, JoinType::JoinInner) && !n.is_natural {
        // For INNER JOIN without qualifications (converted from CROSS JOIN), add ON TRUE
        // This is semantically equivalent to CROSS JOIN
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::TRUE_KW);
    }

    // Emit join alias if present
    // Format: (t1 JOIN t2 ON ...) AS alias (col1, col2, ...)
    if let Some(ref alias) = n.alias {
        e.token(TokenKind::R_PAREN);
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &alias.aliasname);

        // Emit column aliases if present
        if !alias.colnames.is_empty() {
            e.space();
            emit_parenthesized_list(e, &alias.colnames, |emitter, node| {
                if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                    emit_identifier_maybe_quoted(emitter, &s.sval);
                } else {
                    emit_node(emitter, node);
                }
            });
        }
    }

    e.group_end();
}
