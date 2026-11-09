use pgls_query::protobuf::{SelectStmt, SubLink, SubLinkType};

use crate::emitter::{EventEmitter, GroupKind, LineType};
use crate::{Layout, TokenKind};

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
            } else {
                e.token(TokenKind::ANY_KW);
            }
            e.space();

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

fn emit_subquery(e: &mut EventEmitter, node: &pgls_query::protobuf::Node) {
    // Add indentation inside subquery parentheses
    e.line(LineType::Soft);
    e.indent_start();
    if let Some(pgls_query::NodeEnum::SelectStmt(select_stmt)) = node.node.as_ref() {
        // Expanded layout breaks between clauses. A subquery made of its select list alone, such
        // as `(select auth.uid())`, has none to break between, so it only breaks past the width.
        // A subquery with clauses nested in such a subquery, `(select coalesce((select ... from
        // ...), 0))` for instance, uses the configured layout again.
        let layout = if has_select_list_only(select_stmt) {
            Layout::Fit
        } else {
            e.configured_layout()
        };
        e.with_layout(layout, |e| {
            super::emit_select_stmt(e, select_stmt);
        });
    } else {
        // For other node types (e.g., VALUES), emit normally
        super::emit_node(node, e);
    }
    e.indent_end();
    e.line(LineType::Soft);
}

fn has_select_list_only(n: &SelectStmt) -> bool {
    n.into_clause.is_none()
        && n.from_clause.is_empty()
        && n.where_clause.is_none()
        && n.group_clause.is_empty()
        && n.having_clause.is_none()
        && n.window_clause.is_empty()
        && n.values_lists.is_empty()
        && n.sort_clause.is_empty()
        && n.limit_offset.is_none()
        && n.limit_count.is_none()
        && n.locking_clause.is_empty()
        && n.with_clause.is_none()
        && n.larg.is_none()
        && n.rarg.is_none()
}

fn emit_operator_from_list(e: &mut EventEmitter, oper_name: &[pgls_query::protobuf::Node]) {
    // The operator name is typically stored as a list of String nodes
    // For most operators it's just one element like "=" or "<"
    // For qualified operators like "pg_catalog.=" it could be multiple
    if oper_name.is_empty() {
        return;
    }

    // For simplicity, just take the last element which is usually the operator symbol
    if let Some(last) = oper_name.last() {
        if let Some(pgls_query::NodeEnum::String(s)) = last.node.as_ref() {
            e.token(TokenKind::IDENT(s.sval.clone()));
        } else {
            super::emit_node(last, e);
        }
    }
}
