use pgls_query::protobuf::{OnConflictAction, OnConflictClause};

use super::prelude::*;

pub(super) fn emit_on_conflict_clause(e: &mut EventEmitter, n: &OnConflictClause) {
    e.line(LineType::SoftOrSpace);
    e.group_start();

    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::CONFLICT_KW);

    if let Some(ref infer) = n.infer {
        e.line(LineType::SoftOrSpace);
        super::emit_infer_clause(e, infer);
    }

    e.line(LineType::SoftOrSpace);
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

            if !n.target_list.is_empty() {
                e.space();
                emit_set_clause_list(e, &n.target_list);
            }

            if let Some(ref where_clause) = n.where_clause {
                e.line(crate::emitter::LineType::SoftOrSpace);
                e.token(TokenKind::WHERE_KW);
                emit_clause_condition(e, where_clause);
            }
        }
        OnConflictAction::OnconflictNone | OnConflictAction::Undefined => {}
    }

    e.group_end();
}
