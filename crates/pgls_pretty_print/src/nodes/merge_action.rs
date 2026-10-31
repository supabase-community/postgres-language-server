use pgls_query::protobuf::{CmdType, MergeAction, MergeMatchKind};

use super::prelude::*;

pub(super) fn emit_merge_action(e: &mut EventEmitter, action: &MergeAction) {
    e.group_start();

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

    emit_identifier(e, &format!("merge_action#{match_kind}_{command}"));

    if let Some(ref qual) = action.qual {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        emit_clause_condition(e, qual);
    }

    if !action.target_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("TARGET_LIST".to_string()));
        e.space();
        emit_comma_separated_list(e, &action.target_list, emit_node);
    }

    e.group_end();
}
