use pgls_query::protobuf::{CmdType, Query, QuerySource};

use super::prelude::*;

pub(super) fn emit_query(e: &mut EventEmitter, n: &Query) {
    e.group_start(GroupKind::Query);

    if let Some(ref utility) = n.utility_stmt {
        emit_node(utility, e);
        e.group_end();
        return;
    }

    let cmd = match n.command_type() {
        CmdType::CmdSelect => "select",
        CmdType::CmdInsert => "insert",
        CmdType::CmdUpdate => "update",
        CmdType::CmdDelete => "delete",
        CmdType::CmdMerge => "merge",
        CmdType::CmdUtility => "utility",
        CmdType::CmdUnknown | CmdType::Undefined => "unknown",
        CmdType::CmdNothing => "nothing",
    };

    let source = match n.query_source() {
        QuerySource::QsrcOriginal => "original",
        QuerySource::QsrcParser => "parser",
        QuerySource::QsrcInsteadRule => "instead",
        QuerySource::QsrcQualInsteadRule => "qual_instead",
        QuerySource::QsrcNonInsteadRule => "non_instead",
        QuerySource::Undefined => "unspecified",
    };

    emit_identifier(e, &format!("query#{cmd}_{source}"));

    if n.result_relation >= 0 {
        e.space();
        emit_identifier(e, &format!("result_rel#{}", n.result_relation));
    }

    e.group_end();
}
