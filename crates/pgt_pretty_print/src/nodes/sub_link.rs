use pgt_query::protobuf::{SubLink, SubLinkType};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_sub_link(e: &mut EventEmitter, n: &SubLink) {
    e.group_start(GroupKind::SubLink);

    match n.sub_link_type {
        x if x == SubLinkType::ExistsSublink as i32 => {
            // EXISTS(subquery)
            e.token(TokenKind::EXISTS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::AllSublink as i32 => {
            // expr op ALL(subquery)
            if let Some(ref testexpr) = n.testexpr {
                super::emit_node(testexpr, e);
                e.space();

                // Emit operator if present
                if !n.oper_name.is_empty() {
                    emit_operator_from_list(e, &n.oper_name);
                    e.space();
                }
            }

            e.token(TokenKind::ALL_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::AnySublink as i32 => {
            // expr op ANY(subquery) - includes IN which is = ANY
            if let Some(ref testexpr) = n.testexpr {
                super::emit_node(testexpr, e);
                e.space();

                // Special case: empty oper_name means it's IN not = ANY
                if n.oper_name.is_empty() {
                    e.token(TokenKind::IN_KW);
                } else {
                    // Regular ANY with operator
                    emit_operator_from_list(e, &n.oper_name);
                    e.space();
                    e.token(TokenKind::ANY_KW);
                }

                e.space();
            } else {
                e.token(TokenKind::ANY_KW);
                e.space();
            }

            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::RowcompareSublink as i32 => {
            // (expr list) op (subquery)
            if let Some(ref testexpr) = n.testexpr {
                super::emit_node(testexpr, e);
                e.space();

                if !n.oper_name.is_empty() {
                    emit_operator_from_list(e, &n.oper_name);
                    e.space();
                }
            }

            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::ExprSublink as i32 => {
            // Simple scalar subquery: (subquery)
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::MultiexprSublink as i32 => {
            // Multiple expressions - just wrap in parentheses
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::ArraySublink as i32 => {
            // ARRAY(subquery)
            e.token(TokenKind::ARRAY_KW);
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        x if x == SubLinkType::CteSublink as i32 => {
            // For SubPlans only - shouldn't appear in normal SQL
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Fallback to simple subquery
            e.token(TokenKind::L_PAREN);
            if let Some(ref subselect) = n.subselect {
                emit_subquery(e, subselect);
            }
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}

fn emit_subquery(e: &mut EventEmitter, node: &pgt_query::protobuf::Node) {
    // Check if this is a SelectStmt and emit without semicolon
    if let Some(pgt_query::NodeEnum::SelectStmt(select_stmt)) = node.node.as_ref() {
        super::emit_select_stmt_no_semicolon(e, select_stmt);
    } else {
        // For other node types (e.g., VALUES), emit normally
        super::emit_node(node, e);
    }
}

fn emit_operator_from_list(e: &mut EventEmitter, oper_name: &[pgt_query::protobuf::Node]) {
    // The operator name is typically stored as a list of String nodes
    // For most operators it's just one element like "=" or "<"
    // For qualified operators like "pg_catalog.=" it could be multiple
    if oper_name.is_empty() {
        return;
    }

    // For simplicity, just take the last element which is usually the operator symbol
    if let Some(last) = oper_name.last() {
        if let Some(pgt_query::NodeEnum::String(s)) = last.node.as_ref() {
            e.token(TokenKind::IDENT(s.sval.clone()));
        } else {
            super::emit_node(last, e);
        }
    }
}
