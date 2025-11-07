use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::{OnConflictAction, OnConflictExpr};

pub(super) fn emit_on_conflict_expr(e: &mut EventEmitter, n: &OnConflictExpr) {
    e.group_start(GroupKind::OnConflictExpr);

    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::CONFLICT_KW);

    if !n.arbiter_elems.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.arbiter_elems, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    if n.constraint != 0 {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::CONSTRAINT_KW);
        e.space();
        super::emit_identifier(e, &format!("constraint#{}", n.constraint));
    }

    if let Some(ref arbiter_where) = n.arbiter_where {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, arbiter_where);
    }

    e.space();
    e.token(TokenKind::DO_KW);

    match n.action() {
        OnConflictAction::OnconflictNothing => {
            e.space();
            e.token(TokenKind::NOTHING_KW);
        }
        OnConflictAction::OnconflictUpdate => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::UPDATE_KW);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::SET_KW);
            if !n.on_conflict_set.is_empty() {
                e.space();
                super::res_target::emit_set_clause_list(e, &n.on_conflict_set);
            }

            if let Some(ref where_clause) = n.on_conflict_where {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WHERE_KW);
                super::emit_clause_condition(e, where_clause);
            }
        }
        OnConflictAction::OnconflictNone | OnConflictAction::Undefined => {
            e.space();
            super::emit_identifier(e, "on_conflict#undefined");
        }
    }

    if n.excl_rel_index >= 0 && !n.excl_rel_tlist.is_empty() {
        e.line(LineType::SoftOrSpace);
        super::emit_identifier(e, &format!("excluded_relation#{}", n.excl_rel_index));
        e.space();
        super::node_list::emit_comma_separated_list(e, &n.excl_rel_tlist, super::emit_node);
    }

    e.group_end();
}
