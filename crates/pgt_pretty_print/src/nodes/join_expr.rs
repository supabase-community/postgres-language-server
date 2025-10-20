use pgt_query::protobuf::{JoinExpr, JoinType};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::emit_comma_separated_list;
use super::string::emit_identifier;

pub(super) fn emit_join_expr(e: &mut EventEmitter, n: &JoinExpr) {
    e.group_start(GroupKind::JoinExpr);

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
            emit_join_token(TokenKind::IDENT("SEMI".to_string()), e);
        }
        JoinType::JoinAnti => {
            emit_join_token(TokenKind::IDENT("ANTI".to_string()), e);
        }
        JoinType::JoinRightAnti => {
            emit_join_token(TokenKind::RIGHT_KW, e);
            emit_join_token(TokenKind::IDENT("ANTI".to_string()), e);
        }
        JoinType::JoinUniqueOuter | JoinType::JoinUniqueInner | JoinType::Undefined => {
            emit_join_token(TokenKind::CROSS_KW, e);
        }
    }

    emit_join_token(TokenKind::JOIN_KW, e);

    // Right side
    if let Some(ref rarg) = n.rarg {
        e.space();
        super::emit_node(rarg, e);
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
                if let Some(pgt_query::NodeEnum::String(s)) = node.node.as_ref() {
                    emit_identifier(e, &s.sval);
                } else {
                    super::emit_node(node, e);
                }
            });
            e.indent_end();
        } else {
            emit_comma_separated_list(e, &n.using_clause, |node, e| {
                if let Some(pgt_query::NodeEnum::String(s)) = node.node.as_ref() {
                    emit_identifier(e, &s.sval);
                } else {
                    super::emit_node(node, e);
                }
            });
        }
        e.token(TokenKind::R_PAREN);
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

    e.group_end();
}
