use pgt_query::protobuf::{OnConflictAction, OnConflictClause};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::{node_list::emit_comma_separated_list, res_target::emit_set_clause};

pub(super) fn emit_on_conflict_clause(e: &mut EventEmitter, n: &OnConflictClause) {
    e.space();
    e.group_start(GroupKind::OnConflictClause);

    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::CONFLICT_KW);

    if let Some(ref infer) = n.infer {
        e.space();
        super::emit_infer_clause(e, infer);
    }

    e.space();
    e.token(TokenKind::DO_KW);

    match n.action() {
        OnConflictAction::OnconflictNothing => {
            e.space();
            e.token(TokenKind::NOTHING_KW);
        }
        OnConflictAction::OnconflictUpdate => {
            e.space();
            e.token(TokenKind::UPDATE_KW);
            e.space();
            e.token(TokenKind::SET_KW);

            if !n.target_list.is_empty() {
                e.space();
                emit_comma_separated_list(e, &n.target_list, |node, emitter| {
                    emit_set_clause(emitter, assert_node_variant!(ResTarget, node))
                });
            }

            if let Some(ref where_clause) = n.where_clause {
                e.space();
                e.token(TokenKind::WHERE_KW);
                e.space();
                super::emit_node(where_clause, e);
            }
        }
        OnConflictAction::OnconflictNone | OnConflictAction::Undefined => {
            assert!(false, "unexpected OnConflictAction: {:?}", n.action());
        }
    }

    e.group_end();
}
