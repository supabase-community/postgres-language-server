use crate::emitter::{EventEmitter, GroupKind, LineType};
use crate::nodes::node_list::emit_comma_separated_list;
use pgls_query::protobuf::{JoinType, RangeTblEntry, RteKind};
use std::convert::TryFrom;

pub(super) fn emit_range_tbl_entry(e: &mut EventEmitter, n: &RangeTblEntry) {
    e.group_start(GroupKind::RangeTblEntry);

    let kind = RteKind::try_from(n.rtekind).unwrap_or_else(|_| {
        debug_assert!(false, "Unexpected RangeTblEntry rtekind: {}", n.rtekind);
        RteKind::RtekindUndefined
    });

    let mut label = format_label(n, kind);
    if n.security_barrier {
        label.push_str("+barrier");
    }
    if n.lateral {
        label.push_str("+lateral");
    }

    super::emit_identifier(e, &label);

    if let Some(alias) = preferred_alias(n)
        && !alias.is_empty()
    {
        e.space();
        super::emit_identifier_maybe_quoted(e, alias);
    }

    if let Some(tablesample) = n.tablesample.as_deref() {
        e.line(LineType::SoftOrSpace);
        super::table_sample_clause::emit_table_sample_clause(e, tablesample);
    }

    if let Some(subquery) = n.subquery.as_ref() {
        e.line(LineType::SoftOrSpace);
        super::emit_query(e, subquery);
    }

    if let Some(tablefunc) = n.tablefunc.as_ref() {
        e.line(LineType::SoftOrSpace);
        super::table_func::emit_table_func(e, tablefunc);
    }

    if !n.functions.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.functions, super::emit_node);
        if n.funcordinality {
            e.space();
            super::emit_identifier_maybe_quoted(e, "ordinality");
        }
    }

    if !n.values_lists.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.values_lists, super::emit_node);
    }

    if !n.security_quals.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.security_quals, super::emit_node);
    }

    if !n.joinaliasvars.is_empty() && matches!(kind, RteKind::RteJoin) {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.joinaliasvars, super::emit_node);
    }

    e.group_end();
}

fn preferred_alias(entry: &RangeTblEntry) -> Option<&str> {
    entry
        .alias
        .as_ref()
        .and_then(|alias| (!alias.aliasname.is_empty()).then_some(alias.aliasname.as_str()))
        .or_else(|| {
            entry
                .eref
                .as_ref()
                .and_then(|alias| (!alias.aliasname.is_empty()).then_some(alias.aliasname.as_str()))
        })
}

fn format_label(entry: &RangeTblEntry, kind: RteKind) -> String {
    match kind {
        RteKind::RteRelation => {
            let mut base = if entry.relid != 0 {
                format!("rte#rel{}", entry.relid)
            } else {
                "rte#relation".to_string()
            };

            if entry.inh {
                base.push_str("+inh");
            }

            if !entry.relkind.is_empty() {
                base.push('[');
                base.push_str(entry.relkind.as_str());
                base.push(']');
            }

            base
        }
        RteKind::RteSubquery => "rte#subquery".to_string(),
        RteKind::RteJoin => {
            let join = JoinType::try_from(entry.jointype).unwrap_or(JoinType::Undefined);
            format!("rte#join({})", join_label(join))
        }
        RteKind::RteFunction => "rte#function".to_string(),
        RteKind::RteTablefunc => "rte#tablefunc".to_string(),
        RteKind::RteValues => "rte#values".to_string(),
        RteKind::RteCte => {
            if entry.ctename.is_empty() {
                "rte#cte".to_string()
            } else if entry.ctelevelsup == 0 {
                format!("rte#cte({})", entry.ctename)
            } else {
                format!("rte#cte({}^{})", entry.ctename, entry.ctelevelsup)
            }
        }
        RteKind::RteNamedtuplestore => {
            if entry.enrname.is_empty() {
                "rte#tuplestore".to_string()
            } else {
                format!("rte#tuplestore({})", entry.enrname)
            }
        }
        RteKind::RteResult => "rte#result".to_string(),
        RteKind::RtekindUndefined => "rte#unknown".to_string(),
    }
}

fn join_label(join: JoinType) -> &'static str {
    match join {
        JoinType::JoinInner => "inner",
        JoinType::JoinLeft => "left",
        JoinType::JoinFull => "full",
        JoinType::JoinRight => "right",
        JoinType::JoinSemi => "semi",
        JoinType::JoinAnti => "anti",
        JoinType::JoinRightAnti => "right_anti",
        JoinType::JoinUniqueOuter => "unique_outer",
        JoinType::JoinUniqueInner => "unique_inner",
        JoinType::Undefined => "unspecified",
    }
}
