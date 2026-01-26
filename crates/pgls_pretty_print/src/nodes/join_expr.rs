use pgls_query::protobuf::{JoinExpr, JoinType};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::emit_comma_separated_list;
use super::string::emit_identifier;

pub(super) fn emit_join_expr(e: &mut EventEmitter, n: &JoinExpr) {
    e.group_start(GroupKind::JoinExpr);

    // If the join has an alias, we need to wrap it in parentheses
    let has_alias = n.alias.is_some();
    if has_alias {
        e.token(TokenKind::L_PAREN);
    }

    // Left side
    if let Some(ref larg) = n.larg {
        super::emit_node(larg, e);
    }

    if n.larg.is_some() {
        e.line(LineType::SoftOrSpace);
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
        super::emit_node(rarg, e);
        e.indent_end();
    }

    // Join qualification
    if !n.using_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        if n.using_clause.len() > 1 {
            e.indent_start();
            e.line(LineType::SoftOrSpace);
            emit_comma_separated_list(e, &n.using_clause, |node, e| {
                // For USING clause, String nodes should be identifiers
                if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                    emit_identifier(e, &s.sval);
                } else {
                    super::emit_node(node, e);
                }
            });
            e.indent_end();
        } else {
            emit_comma_separated_list(e, &n.using_clause, |node, e| {
                if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                    emit_identifier(e, &s.sval);
                } else {
                    super::emit_node(node, e);
                }
            });
        }
        e.token(TokenKind::R_PAREN);

        // Emit USING alias if present (PostgreSQL 14+)
        // Format: USING (col1, col2) AS alias
        if let Some(ref using_alias) = n.join_using_alias {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            emit_identifier(e, &using_alias.aliasname);
        }
    } else if let Some(ref quals) = n.quals {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        e.indent_start();
        super::emit_node(quals, e);
        e.indent_end();
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
        emit_identifier(e, &alias.aliasname);

        // Emit column aliases if present
        if !alias.colnames.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &alias.colnames, |node, emitter| {
                if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                    emit_identifier(emitter, &s.sval);
                } else {
                    super::emit_node(node, emitter);
                }
            });
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}
