use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::{CmdType, MergeAction, MergeMatchKind, MergeWhenClause};

use super::res_target::emit_set_clause_list;

pub(super) fn emit_merge_when_clause(e: &mut EventEmitter, clause: &MergeWhenClause) {
    e.group_start(GroupKind::MergeWhenClause);

    e.token(TokenKind::WHEN_KW);
    e.space();

    match clause.match_kind() {
        MergeMatchKind::MergeWhenMatched => {
            e.token(TokenKind::MATCHED_KW);
        }
        MergeMatchKind::MergeWhenNotMatchedBySource => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::MATCHED_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            e.token(TokenKind::IDENT("SOURCE".to_string()));
        }
        MergeMatchKind::MergeWhenNotMatchedByTarget => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::MATCHED_KW);
            if clause.condition.is_none() {
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                e.token(TokenKind::IDENT("TARGET".to_string()));
            }
        }
        MergeMatchKind::Undefined => {}
    }

    if let Some(ref cond) = clause.condition {
        e.space();
        e.token(TokenKind::AND_KW);
        super::emit_clause_condition(e, cond);
    }

    e.space();
    e.token(TokenKind::THEN_KW);
    e.space();

    match clause.command_type() {
        CmdType::CmdUpdate => {
            e.token(TokenKind::UPDATE_KW);
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            emit_set_clause_list(e, &clause.target_list);
        }
        CmdType::CmdInsert => {
            e.token(TokenKind::INSERT_KW);

            if !clause.target_list.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                super::node_list::emit_comma_separated_list(
                    e,
                    &clause.target_list,
                    |node, emitter| {
                        let target = assert_node_variant!(ResTarget, node);
                        if !target.name.is_empty() {
                            super::emit_identifier_maybe_quoted(emitter, &target.name);
                        }
                    },
                );
                e.token(TokenKind::R_PAREN);
            }

            if !clause.values.is_empty() {
                e.space();
                e.token(TokenKind::VALUES_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                super::node_list::emit_comma_separated_list(e, &clause.values, super::emit_node);
                e.token(TokenKind::R_PAREN);
            } else {
                e.space();
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                e.token(TokenKind::VALUES_KW);
            }
        }
        CmdType::CmdDelete => {
            e.token(TokenKind::DELETE_KW);
        }
        CmdType::Undefined | CmdType::CmdUnknown => {
            e.token(TokenKind::DO_KW);
            e.space();
            e.token(TokenKind::IDENT("NOTHING".to_string()));
        }
        _ => {
            e.token(TokenKind::DO_KW);
            e.space();
            e.token(TokenKind::IDENT("NOTHING".to_string()));
        }
    }

    e.group_end();
}

pub(super) fn emit_merge_action(e: &mut EventEmitter, action: &MergeAction) {
    e.group_start(GroupKind::MergeAction);

    let match_kind = match action.match_kind() {
        MergeMatchKind::MergeWhenMatched => "matched",
        MergeMatchKind::MergeWhenNotMatchedByTarget => "not_target",
        MergeMatchKind::MergeWhenNotMatchedBySource => "not_source",
        MergeMatchKind::Undefined => "unspecified",
    };

    let command = match action.command_type() {
        CmdType::CmdInsert => "insert",
        CmdType::CmdUpdate => "update",
        CmdType::CmdDelete => "delete",
        CmdType::Undefined | CmdType::CmdUnknown => "none",
        _ => "other",
    };

    super::emit_identifier(e, &format!("merge_action#{}_{}", match_kind, command));

    if let Some(ref qual) = action.qual {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, qual);
    }

    if !action.target_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("TARGET_LIST".to_string()));
        e.space();
        super::node_list::emit_comma_separated_list(e, &action.target_list, super::emit_node);
    }

    e.group_end();
}
