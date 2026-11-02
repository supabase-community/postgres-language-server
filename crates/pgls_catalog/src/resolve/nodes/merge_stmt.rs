use pgls_query::{
    Node, NodeEnum,
    protobuf::{CmdType, MergeMatchKind, MergeStmt, MergeSupportFunc, MergeWhenClause},
};

use super::{
    Resolver,
    from_clause::resolve_from_clause,
    insert_stmt::{check_insert_width, insert_target_types, node_location},
    range_var::resolve_target_relation,
    res_target::{check_target_column, resolve_returning_clause, resolve_set_clause},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::{
    FindingKind, MergeError,
    expr::{check_assignment, infer_expr},
    scope::{Columns, Item, Level},
};

/// Resolves a MERGE and returns the columns of its RETURNING list. Port of
/// [`transformMergeStmt`].
///
/// [`transformMergeStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_merge.c#L107
pub(super) fn resolve_merge_stmt(r: &mut Resolver, n: &MergeStmt) -> Columns {
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        if with.recursive {
            r.report(
                FindingKind::InvalidMerge(MergeError::RecursiveWith),
                with.location,
            );
        }
        resolve_with_clause(r, with);
    }
    let columns = resolve_merge(r, n);
    r.ctes.truncate(ctes);
    columns
}

fn resolve_merge(r: &mut Resolver, n: &MergeStmt) -> Columns {
    let Some(relation) = &n.relation else {
        r.depends_on_file();
        return None;
    };
    check_reachable_when_clauses(r, &n.merge_when_clauses);
    // The target is never a CTE (`setTargetTable`).
    let target = resolve_target_relation(r, relation);
    // The source is transformed like a FROM list of one item, before the target is in the
    // namespace, so a LATERAL source can't see the target.
    let source = match n.source_relation.as_deref() {
        Some(source) => resolve_from_clause(r, std::slice::from_ref(source)),
        None => {
            r.depends_on_file();
            Level::default()
        }
    };
    if let Some(source) = n.source_relation.as_deref() {
        check_distinct_names(r, relation, source);
    }
    let mut both = Level::with_item(target.clone());
    both.extend(source.clone());

    // The join condition sees the source and the target.
    if let Some(condition) = n.join_condition.as_deref() {
        r.enter_level(both.clone());
        resolve_expr(r, condition);
        r.exit_level();
    }

    for clause in &n.merge_when_clauses {
        match clause.node.as_ref() {
            Some(NodeEnum::MergeWhenClause(clause)) => {
                resolve_when_clause(r, clause, &target, &source);
            }
            _ => r.depends_on_file(),
        }
    }

    // RETURNING sees the source and the target, and `old` and `new` for the target row. The
    // first item of the level is the target.
    let in_merge_returning = std::mem::replace(&mut r.in_merge_returning, true);
    let columns = resolve_returning_clause(r, n.returning_clause.as_ref(), both);
    r.in_merge_returning = in_merge_returning;
    columns
}

/// Reports the first `WHEN` clause after one of the same kind without condition, which matches
/// every row first. Postgres gives no position.
fn check_reachable_when_clauses(r: &mut Resolver, clauses: &[Node]) {
    let mut terminal = Vec::new();
    for clause in clauses {
        let Some(NodeEnum::MergeWhenClause(clause)) = clause.node.as_ref() else {
            continue;
        };
        let kind = clause.match_kind();
        if terminal.contains(&kind) {
            r.report(FindingKind::InvalidMerge(MergeError::UnreachableWhen), -1);
            return;
        }
        if clause.condition.is_none() {
            terminal.push(kind);
        }
    }
}

/// Reports a target and a source with the same name, which Postgres checks by the names of
/// their namespace items. The name of an unaliased subquery, function, or join is a placeholder
/// or not certain, so only relations, CTEs, and aliased items are compared.
fn check_distinct_names(r: &mut Resolver, target: &pgls_query::protobuf::RangeVar, source: &Node) {
    let target_name = target
        .alias
        .as_ref()
        .map_or(target.relname.as_str(), |alias| alias.aliasname.as_str());
    let (source_name, location) = match source.node.as_ref() {
        Some(NodeEnum::RangeVar(source)) => (
            source
                .alias
                .as_ref()
                .map_or(source.relname.as_str(), |alias| alias.aliasname.as_str()),
            source.location,
        ),
        Some(NodeEnum::RangeSubselect(source)) => match &source.alias {
            Some(alias) => (alias.aliasname.as_str(), -1),
            None => return,
        },
        Some(NodeEnum::RangeFunction(source)) => match &source.alias {
            Some(alias) => (alias.aliasname.as_str(), -1),
            None => return,
        },
        // `(a join b) as j`, or `a join b using (id) as j`.
        Some(NodeEnum::JoinExpr(source)) => {
            match source.alias.as_ref().or(source.join_using_alias.as_ref()) {
                Some(alias) => (alias.aliasname.as_str(), -1),
                None => return,
            }
        }
        _ => return,
    };
    if source_name == target_name {
        r.report(
            FindingKind::InvalidMerge(MergeError::DuplicateName {
                name: target_name.to_owned(),
            }),
            location,
        );
    }
}

/// `merge_action()` is only allowed in the RETURNING list of MERGE, and in its subqueries
/// ([`transformMergeSupportFunc`]).
///
/// [`transformMergeSupportFunc`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1388
pub(super) fn check_merge_action(r: &mut Resolver, n: &MergeSupportFunc) {
    if !r.in_merge_returning {
        r.report(
            FindingKind::InvalidMerge(MergeError::MergeActionOutsideReturning),
            n.location,
        );
    }
}

/// A `WHEN [NOT] MATCHED` clause, with what its kind can see ([`setNamespaceForMergeWhen`]):
/// `MATCHED` sees the source and the target, `NOT MATCHED BY SOURCE` only the target, and
/// `NOT MATCHED [BY TARGET]` only the source. The INSERT target columns are those of the target
/// anyway.
///
/// [`setNamespaceForMergeWhen`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_merge.c#L52
fn resolve_when_clause(r: &mut Resolver, n: &MergeWhenClause, target: &Item, source: &Level) {
    let level = match n.match_kind() {
        MergeMatchKind::MergeWhenMatched => {
            let mut both = Level::with_item(target.clone());
            both.extend(source.clone());
            both
        }
        MergeMatchKind::MergeWhenNotMatchedBySource => Level::with_item(target.clone()),
        MergeMatchKind::MergeWhenNotMatchedByTarget => source.clone(),
        MergeMatchKind::Undefined => {
            r.depends_on_file();
            return;
        }
    };
    r.enter_level(level);

    if let Some(condition) = n.condition.as_deref() {
        resolve_expr(r, condition);
    }
    match n.command_type() {
        CmdType::CmdInsert => resolve_insert_action(r, n, target),
        CmdType::CmdUpdate => resolve_set_clause(r, target, &n.target_list),
        CmdType::CmdDelete | CmdType::CmdNothing => {}
        _ => r.depends_on_file(),
    }

    r.exit_level();
}

/// `INSERT [(columns)] VALUES (...)` or `INSERT DEFAULT VALUES`, like the single-row case of
/// [`transformInsertStmt`]: the target columns ([`checkInsertTargets`]), the number of values,
/// and the assignment of each value ([`transformInsertRow`]).
///
/// [`transformInsertStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L633
/// [`checkInsertTargets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1018
/// [`transformInsertRow`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1060
fn resolve_insert_action(r: &mut Resolver, n: &MergeWhenClause, target: &Item) {
    let mut target_columns = Vec::new();
    let mut partial_targets = Vec::new();
    for column in &n.target_list {
        let Some(NodeEnum::ResTarget(column)) = &column.node else {
            r.depends_on_file();
            continue;
        };
        check_target_column(r, target, &column.name, column.location);
        target_columns.push(column.name.clone());
        partial_targets.push(!column.indirection.is_empty());
    }

    // `DEFAULT VALUES` has no values.
    if n.values.is_empty() {
        return;
    }
    let target_types = insert_target_types(target, &target_columns, &partial_targets);
    for (index, value) in n.values.iter().enumerate() {
        resolve_node(r, value);
        let Some(expr) = value.node.as_ref() else {
            continue;
        };
        let value_type = infer_expr(r, expr);
        if let Some(Some(column)) = target_types.get(index)
            && let Some(column_type) = column.ty.clone()
        {
            let location = node_location(value).unwrap_or_default();
            check_assignment(
                r,
                &column.name,
                column_type,
                Some(expr),
                value_type,
                location,
            );
        }
    }
    check_insert_width(r, target, &target_columns, n.values.len());
}

/// Resolves the names of a condition and types it, which reports its operators and functions.
fn resolve_expr(r: &mut Resolver, node: &Node) {
    resolve_node(r, node);
    if let Some(expr) = node.node.as_ref() {
        infer_expr(r, expr);
    }
}
