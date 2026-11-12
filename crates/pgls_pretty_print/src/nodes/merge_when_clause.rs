use pgls_query::protobuf::{CmdType, MergeMatchKind, MergeWhenClause, OverridingKind};

use super::prelude::*;

pub(super) fn emit_merge_when_clause(e: &mut EventEmitter, clause: &MergeWhenClause) {
    e.group_start();

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
            e.token(TokenKind::SOURCE_KW);
        }
        MergeMatchKind::MergeWhenNotMatchedByTarget => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::MATCHED_KW);
            if clause.condition.is_none() {
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                e.token(TokenKind::TARGET_KW);
            }
        }
        MergeMatchKind::Undefined => {}
    }

    if let Some(ref cond) = clause.condition {
        e.space();
        e.token(TokenKind::AND_KW);
        emit_clause_condition(e, cond);
    }

    e.line(LineType::SoftOrSpace);
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

            // Column list - wrap in group for compact formatting
            if !clause.target_list.is_empty() {
                e.space();
                e.group_start();
                e.token(TokenKind::L_PAREN);
                e.line(LineType::Soft);
                e.indent_start();
                emit_comma_separated_list(e, &clause.target_list, |emitter, node| {
                    let target = assert_node_variant!(ResTarget, node);
                    if !target.name.is_empty() {
                        emit_identifier_maybe_quoted(emitter, &target.name);
                    }
                });
                e.indent_end();
                e.line(LineType::Soft);
                e.token(TokenKind::R_PAREN);
                e.group_end();
            }

            match clause.r#override() {
                OverridingKind::OverridingUserValue => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::OVERRIDING_KW);
                    e.space();
                    e.token(TokenKind::USER_KW);
                    e.space();
                    e.token(TokenKind::VALUE_KW);
                }
                OverridingKind::OverridingSystemValue => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::OVERRIDING_KW);
                    e.space();
                    e.token(TokenKind::SYSTEM_KW);
                    e.space();
                    e.token(TokenKind::VALUE_KW);
                }
                OverridingKind::OverridingNotSet | OverridingKind::Undefined => {}
            }

            // Values list - wrap in group for compact formatting
            if !clause.values.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::VALUES_KW);
                e.space();
                e.group_start();
                e.token(TokenKind::L_PAREN);
                e.line(LineType::Soft);
                e.indent_start();
                emit_comma_separated_list(e, &clause.values, emit_node);
                e.indent_end();
                e.line(LineType::Soft);
                e.token(TokenKind::R_PAREN);
                e.group_end();
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
            e.token(TokenKind::NOTHING_KW);
        }
        _ => {
            e.token(TokenKind::DO_KW);
            e.space();
            e.token(TokenKind::NOTHING_KW);
        }
    }

    e.group_end();
}
