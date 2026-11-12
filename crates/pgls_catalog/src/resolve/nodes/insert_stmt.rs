use pgls_query::{
    Node, NodeEnum,
    protobuf::{InsertStmt, OverridingKind, ResTarget, SelectStmt, SetOperation},
};

use super::{
    Resolver,
    on_conflict_clause::resolve_on_conflict_clause,
    range_var::resolve_target_relation,
    res_target::{check_target_column, resolve_returning_clause},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::{
    FindingKind,
    column_name::leftmost_select,
    expr::{check_assignment, infer_expr},
    scope::{Columns, DefaultOnly, Item, Level},
    utils::node_location,
};
use crate::typing::{Type, TypedColumn};
use pgls_text_size::TextRange;

/// Resolves an INSERT and returns the columns of its RETURNING list. Port of
/// [`transformInsertStmt`]: the target columns ([`checkInsertTargets`]), the number of
/// values, and the assignment of each value ([`transformInsertRow`]).
///
/// [`transformInsertStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L633
/// [`checkInsertTargets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1018
/// [`transformInsertRow`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1060
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

    let target_columns = insert_target_columns(r, &target, &n.cols);

    // The number of columns of the source query, if known.
    let mut source_width = None;
    if let Some(source) = n.select_stmt.as_deref() {
        resolve_node(r, source);
        let source_types = r.output.clone();
        source_width = source_types.as_ref().map(Vec::len);

        let target_types = insert_target_types(&target, &target_columns);
        let check = |r: &mut Resolver,
                     expr: Option<&NodeEnum>,
                     value: Option<Type>,
                     target: Option<&TypedColumn>,
                     location: i32| {
            if let Some(target) = target
                && let Some(column_type) = target.ty.clone()
            {
                check_assignment(r, &target.name, column_type, expr, value, location);
            }
        };
        match values_rows(source) {
            // Each row of `INSERT ... VALUES` is coerced to the target columns on its own.
            Some(rows) => {
                for row in rows {
                    for (value, target) in row.iter().zip(&target_types) {
                        let location = node_location(value).unwrap_or_default();
                        let Some(value) = value.node.as_ref() else {
                            continue;
                        };
                        let value_type = infer_expr(r, value);
                        check(r, Some(value), value_type, *target, location);
                    }
                }
            }
            None => {
                for (index, (value, target)) in source_types
                    .unwrap_or_default()
                    .iter()
                    .zip(&target_types)
                    .enumerate()
                {
                    let location = select_target_location(source, index).unwrap_or_default();
                    let expr = select_value(source, index);
                    check(r, expr, value.ty.clone(), *target, location);
                }
            }
        }
        let found = match &source.node {
            Some(NodeEnum::SelectStmt(select)) => select_width(select),
            _ => None,
        };
        if let Some(found) = found {
            check_insert_width(r, &target, &target_columns, found);
        }
    }

    // `DEFAULT VALUES` assigns nothing.
    if let Some(source) = n.select_stmt.as_deref() {
        let values = match values_rows(source) {
            Some(rows) => InsertValues::Rows(rows),
            None => InsertValues::Query {
                source,
                width: source_width,
            },
        };
        check_generated_insert(r, &target, &target_columns, n.r#override(), values);
    }

    if let Some(conflict) = &n.on_conflict_clause {
        resolve_on_conflict_clause(r, conflict, &target);
    }

    resolve_returning_clause(
        r,
        n.returning_clause.as_ref(),
        &target,
        Level::with_item(target.clone()),
    )
}

/// The entries of the column list of an INSERT, each checked against `target`
/// ([`checkInsertTargets`]).
///
/// [`checkInsertTargets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1018
pub(super) fn insert_target_columns<'a>(
    r: &mut Resolver,
    target: &Item,
    columns: &'a [Node],
) -> Vec<&'a ResTarget> {
    let mut targets = Vec::new();
    for column in columns {
        let Some(NodeEnum::ResTarget(column)) = &column.node else {
            r.depends_on_file();
            continue;
        };
        check_target_column(r, target, &column.name, column.location);
        targets.push(column.as_ref());
    }
    targets
}

/// The type of each target column of an INSERT, in the order of the values. `None` for columns
/// whose type is unknown and for targets with subscripts or fields, like `a[1]` or `c.f`, which
/// assign to part of a column.
pub(super) fn insert_target_types<'a>(
    target: &'a Item,
    target_columns: &[&ResTarget],
) -> Vec<Option<&'a TypedColumn>> {
    let targets = target.typed_columns.as_deref().unwrap_or_default();
    if target_columns.is_empty() {
        targets.iter().map(Some).collect()
    } else {
        target_columns
            .iter()
            .map(|target| {
                target
                    .indirection
                    .is_empty()
                    .then(|| targets.iter().find(|column| column.name == target.name))
                    .flatten()
            })
            .collect()
    }
}

/// Reports an INSERT with more values than target columns, or, with a column list, fewer
/// ([`transformInsertRow`]).
///
/// [`transformInsertRow`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1060
pub(super) fn check_insert_width(
    r: &mut Resolver,
    target: &Item,
    target_columns: &[&ResTarget],
    found: usize,
) {
    let expected = if target_columns.is_empty() {
        target.columns.as_ref().map(Vec::len)
    } else {
        Some(target_columns.len())
    };
    let Some(expected) = expected else {
        return;
    };
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

/// The values of an INSERT, or of the INSERT action of MERGE.
pub(super) enum InsertValues<'a> {
    /// The rows of a plain `VALUES` list, or the one row of a MERGE action.
    Rows(Vec<&'a [Node]>),
    /// Any other query, with the number of columns it produces, if known.
    Query {
        source: &'a Node,
        width: Option<usize>,
    },
}

/// Reports an INSERT that assigns anything but `DEFAULT` to a generated column, or to a
/// `GENERATED ALWAYS` identity column without `OVERRIDING SYSTEM VALUE` or `OVERRIDING USER
/// VALUE`. Port of the checks of [`rewriteTargetListIU`] for `CMD_INSERT`: in a VALUES list,
/// every row must have `DEFAULT` for the column, and a SELECT can't assign it at all.
/// [`RewriteQuery`] applies them to the INSERT action of MERGE too, with the action's
/// `OVERRIDING` clause ([`transformMergeStmt`]).
///
/// Targets with subscripts or fields, like `g[1]`, are not checked. Without a column list, a
/// SELECT whose `*` expands to unknown columns isn't either.
///
/// [`rewriteTargetListIU`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L808
/// [`RewriteQuery`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L4162
/// [`transformMergeStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_merge.c#L317
pub(super) fn check_generated_insert(
    r: &mut Resolver,
    target: &Item,
    target_columns: &[&ResTarget],
    overriding: OverridingKind,
    values: InsertValues,
) {
    let overriding = matches!(
        overriding,
        OverridingKind::OverridingSystemValue | OverridingKind::OverridingUserValue
    );
    if target.default_only_columns.is_empty() {
        return;
    }

    // The assigned columns and the location of each in the column list.
    let targets: Vec<(&str, Option<i32>)> = if target_columns.is_empty() {
        // Without a column list, the values go to the first columns of the table.
        let width = match &values {
            InsertValues::Rows(rows) => rows.first().map(|row| row.len()),
            InsertValues::Query { source, width } => match &source.node {
                Some(NodeEnum::SelectStmt(select)) => select_width(select).or(*width),
                _ => None,
            },
        };
        let (Some(width), Some(columns)) = (width, target.columns.as_ref()) else {
            return;
        };
        columns
            .iter()
            .take(width)
            .map(|column| (column.as_str(), None))
            .collect()
    } else {
        target_columns
            .iter()
            .map(|column| {
                if column.indirection.is_empty() {
                    (column.name.as_str(), Some(column.location))
                } else {
                    // Not checked: an empty name is never a generated column.
                    ("", None)
                }
            })
            .collect()
    };

    for (index, (column, column_location)) in targets.into_iter().enumerate() {
        let Some(kind) = target.default_only(column, overriding) else {
            continue;
        };
        let span = match &values {
            InsertValues::Rows(rows) => {
                let value = rows.iter().find_map(|row| {
                    row.get(index)
                        .filter(|value| !matches!(value.node, Some(NodeEnum::SetToDefault(_))))
                });
                match value {
                    Some(value) => column_location
                        .or_else(|| node_location(value))
                        .and_then(|location| r.name_span(location)),
                    None => continue,
                }
            }
            InsertValues::Query { source, width } => match column_location {
                Some(location) => r.name_span(location),
                None => select_target_span(r, source, index, *width),
            },
        };
        r.report_with_span(
            FindingKind::GeneratedColumnAssignment {
                column: column.to_owned(),
                insert: true,
                identity: kind == DefaultOnly::IdentityAlways,
            },
            span,
        );
    }
}

fn select_target_location(source: &pgls_query::Node, index: usize) -> Option<i32> {
    let NodeEnum::SelectStmt(select) = source.node.as_ref()? else {
        return None;
    };
    if let Some(row) = select.values_lists.first() {
        let NodeEnum::List(row) = row.node.as_ref()? else {
            return None;
        };
        return node_location(row.items.get(index)?);
    }
    let NodeEnum::ResTarget(target) = select.target_list.get(index)?.node.as_ref()? else {
        return None;
    };
    node_location(target.val.as_deref()?)
}

/// The span of the select list entry that produces the column at `index` of a SELECT with
/// `width` columns: the `*` or `t.*` it is part of, if any. With several `*`, it's not known which
/// one expands to the column.
fn select_target_span(
    r: &Resolver,
    source: &pgls_query::Node,
    index: usize,
    width: Option<usize>,
) -> Option<TextRange> {
    let NodeEnum::SelectStmt(select) = source.node.as_ref()? else {
        return None;
    };
    let values: Vec<&NodeEnum> = select
        .target_list
        .iter()
        .map(|target| match target.node.as_ref()? {
            NodeEnum::ResTarget(target) => target.val.as_deref()?.node.as_ref(),
            _ => None,
        })
        .collect::<Option<_>>()?;
    let mut stars = (0..values.len()).filter(|&entry| expands_to_columns(values[entry]));
    let Some(star) = stars.next() else {
        return r.name_span(select_target_location(source, index)?);
    };
    if stars.next().is_some() {
        return None;
    }
    // The `*` expands to the columns the other entries don't produce.
    let star_width = (width? + 1).checked_sub(values.len())?;
    if (star..star + star_width).contains(&index) {
        let NodeEnum::ColumnRef(column) = values[star] else {
            return None;
        };
        return r.star_span(column.location);
    }
    let entry = if index < star {
        index
    } else {
        index + 1 - star_width
    };
    let NodeEnum::ResTarget(target) = select.target_list.get(entry)?.node.as_ref()? else {
        return None;
    };
    r.name_span(node_location(target.val.as_deref()?)?)
}

/// The value of the select list entry at `index` of a plain SELECT. Postgres doesn't resolve
/// the unknown literals of the source of an INSERT, so they are coerced to the target columns
/// like the values of `INSERT ... VALUES` ([`transformInsertStmt`]). Except where DISTINCT,
/// GROUP BY or ORDER BY sort them, which makes them text ([`addTargetToSortList`]), so `None`
/// for a SELECT with any of them, and for set operations, which resolve them too.
///
/// [`transformInsertStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L771
/// [`addTargetToSortList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L3393
fn select_value(source: &pgls_query::Node, index: usize) -> Option<&NodeEnum> {
    let NodeEnum::SelectStmt(select) = source.node.as_ref()? else {
        return None;
    };
    if select.op() != SetOperation::SetopNone
        || !select.values_lists.is_empty()
        || !select.distinct_clause.is_empty()
        || !select.group_clause.is_empty()
        || !select.sort_clause.is_empty()
    {
        return None;
    }
    let mut values = select
        .target_list
        .iter()
        .map(|target| match target.node.as_ref()? {
            NodeEnum::ResTarget(target) => target.val.as_deref()?.node.as_ref(),
            _ => None,
        });
    // A `*` before the entry expands to any number of columns.
    for _ in 0..index {
        if values.next()?.is_none_or(expands_to_columns) {
            return None;
        }
    }
    values.next()?
}

/// The rows of a plain `VALUES` list.
fn values_rows(source: &pgls_query::Node) -> Option<Vec<&[pgls_query::Node]>> {
    let Some(NodeEnum::SelectStmt(select)) = &source.node else {
        return None;
    };
    if select.op() != SetOperation::SetopNone || select.values_lists.is_empty() {
        return None;
    }
    select
        .values_lists
        .iter()
        .map(|row| match &row.node {
            Some(NodeEnum::List(row)) => Some(row.items.as_slice()),
            _ => None,
        })
        .collect()
}

/// The number of columns a query produces, if it is certain.
/// A set operation has the columns of its left-most select.
fn select_width(select: &SelectStmt) -> Option<usize> {
    let select = leftmost_select(select)?;
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

#[cfg(test)]
mod tests {
    use pgls_query::protobuf::{AConst, Node, ResTarget};

    use super::*;

    /// A chain of `select 1, 2 union select 1, 2 union ...` with this many arms. Recursing once
    /// per arm overflowed the stack below.
    const ARMS: usize = 10_000;
    const SMALL_STACK_BYTES: usize = 256 * 1024;

    fn two_columns() -> SelectStmt {
        let target = || Node {
            node: Some(NodeEnum::ResTarget(Box::new(ResTarget {
                val: Some(Box::new(Node {
                    node: Some(NodeEnum::AConst(AConst::default())),
                })),
                ..Default::default()
            }))),
        };
        SelectStmt {
            target_list: vec![target(), target()],
            op: SetOperation::SetopNone.into(),
            ..Default::default()
        }
    }

    #[test]
    fn long_set_operation_chains_dont_overflow_the_stack() {
        // Dropping the chain recurses, so it is built and dropped on a thread with a large stack.
        std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(|| {
                let mut select = two_columns();
                for _ in 0..ARMS {
                    select = SelectStmt {
                        op: SetOperation::SetopUnion.into(),
                        larg: Some(Box::new(select)),
                        rarg: Some(Box::new(two_columns())),
                        ..Default::default()
                    };
                }
                std::thread::scope(|scope| {
                    std::thread::Builder::new()
                        .stack_size(SMALL_STACK_BYTES)
                        .spawn_scoped(scope, || assert_eq!(select_width(&select), Some(2)))
                        .unwrap()
                        .join()
                        .unwrap();
                });
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
