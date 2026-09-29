use pgls_query::{
    NodeEnum,
    protobuf::{InsertStmt, SelectStmt, SetOperation},
};

use super::{
    Resolver,
    on_conflict_clause::resolve_on_conflict_clause,
    range_var::resolve_target_relation,
    res_target::{check_target_column, resolve_returning_list},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::{
    FindingKind,
    scope::{Columns, Level},
};

/// Resolves an INSERT and returns the columns of its RETURNING list.
pub(super) fn resolve_insert_stmt(r: &mut Resolver, n: &InsertStmt) -> Columns {
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        resolve_with_clause(r, with);
    }
    let columns = resolve_insert(r, n);
    r.ctes.truncate(ctes);
    columns
}

fn resolve_insert(r: &mut Resolver, n: &InsertStmt) -> Columns {
    let Some(relation) = &n.relation else {
        r.depends_on_file();
        return None;
    };
    let target = resolve_target_relation(r, relation);

    let mut target_columns = Vec::new();
    for column in &n.cols {
        let Some(NodeEnum::ResTarget(column)) = &column.node else {
            r.depends_on_file();
            continue;
        };
        check_target_column(r, &target, &column.name, column.location);
        target_columns.push(column.name.clone());
    }

    if let Some(source) = n.select_stmt.as_deref() {
        resolve_node(r, source);
        let found = match &source.node {
            Some(NodeEnum::SelectStmt(select)) => select_width(select),
            _ => None,
        };
        let expected = if target_columns.is_empty() {
            target.columns.as_ref().map(Vec::len)
        } else {
            Some(target_columns.len())
        };
        if let (Some(found), Some(expected)) = (found, expected) {
            // Without a column list, fewer values than columns are fine.
            let mismatch = if target_columns.is_empty() {
                found > expected
            } else {
                found != expected
            };
            if mismatch {
                r.report(FindingKind::InsertColumnMismatch { expected, found }, -1);
            }
        }
    }

    if let Some(conflict) = &n.on_conflict_clause {
        resolve_on_conflict_clause(r, conflict, &target);
    }

    resolve_returning_list(r, &n.returning_list, Level::with_item(target))
}

/// The number of columns a query produces, if it is certain.
fn select_width(select: &SelectStmt) -> Option<usize> {
    if select.op() != SetOperation::SetopNone {
        return select_width(select.larg.as_deref()?);
    }
    if let Some(row) = select.values_lists.first() {
        return match &row.node {
            Some(NodeEnum::List(row)) => Some(row.items.len()),
            _ => None,
        };
    }
    let mut width = 0;
    for target in &select.target_list {
        let NodeEnum::ResTarget(target) = target.node.as_ref()? else {
            return None;
        };
        if expands_to_columns(target.val.as_deref()?.node.as_ref()?) {
            return None;
        }
        width += 1;
    }
    Some(width)
}

/// `*`, `t.*`, and `(row).*` expand to any number of columns.
fn expands_to_columns(value: &NodeEnum) -> bool {
    let last = match value {
        NodeEnum::ColumnRef(column) => column.fields.last(),
        NodeEnum::AIndirection(indirection) => indirection.indirection.last(),
        _ => return false,
    };
    last.is_some_and(|last| matches!(last.node, Some(NodeEnum::AStar(_))))
}
