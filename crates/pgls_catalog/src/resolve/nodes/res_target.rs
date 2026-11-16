//! Select lists, RETURNING lists, and the target columns of INSERT and UPDATE.

use pgls_query::{
    Node, NodeEnum,
    protobuf::{ReturningClause, ReturningOptionKind},
};
use pgls_query_ext::utils::{star_qualifier, string_values};

use super::{Resolver, resolve_list};
use crate::resolve::{
    FindingKind, Reference, StarColumn,
    expr::{check_assignment, infer_expr, infer_indirection},
    output::{QueryOutput, Star, TargetOutput},
    scope::{
        DefaultOnly, Item, Level, StarEntry, find_item, find_qualified_item, has_opaque_items,
    },
    utils::{node_location, row_columns},
};
use crate::typing::TypedColumn;

/// The output columns of a select list or RETURNING list, one entry at a time. Port of the
/// names of [`transformTargetList`], with [`FigureColname`] for unnamed entries,
/// [`ExpandColumnRefStar`] for `*`, `t.*` and `s.t.*`, and [`ExpandIndirectionStar`] for
/// `(expr).*`. The columns of a `*` have the types of their FROM item; the expressions are typed
/// by [`infer_select_list`].
///
/// [`transformTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L121
/// [`FigureColname`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1713
/// [`ExpandColumnRefStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1123
/// [`ExpandIndirectionStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1444
pub(super) fn select_list_output(r: &mut Resolver, targets: &[Node]) -> QueryOutput {
    QueryOutput::new(
        targets
            .iter()
            .map(|target| target_output(r, target))
            .collect(),
    )
}

fn target_output(r: &mut Resolver, target: &Node) -> TargetOutput {
    let Some(NodeEnum::ResTarget(target)) = target.node.as_ref() else {
        return TargetOutput {
            location: -1,
            star: None,
            columns: None,
        };
    };
    let value = target.val.as_deref().and_then(|value| value.node.as_ref());
    if let Some(NodeEnum::ColumnRef(column)) = value
        && let Some(qualifier) = star_qualifier(column).and_then(string_values)
    {
        let columns = expand_column_ref_star(r, &qualifier);
        return TargetOutput {
            location: column.location,
            star: Some(Star::Columns(qualifier)),
            columns,
        };
    }
    if let Some(NodeEnum::AIndirection(indirection)) = value
        && let Some((last, fields)) = indirection.indirection.split_last()
        && matches!(last.node, Some(NodeEnum::AStar(_)))
    {
        return TargetOutput {
            location: target.location,
            star: Some(Star::Fields),
            columns: expand_indirection_star(r, indirection.arg.as_deref(), fields),
        };
    }
    let name = value.and_then(|_| r.target_name(target));
    TargetOutput {
        location: target
            .val
            .as_deref()
            .and_then(node_location)
            .unwrap_or(target.location),
        star: None,
        columns: name.map(|name| vec![TypedColumn { name, ty: None }]),
    }
}

/// The columns that `*` (an empty `qualifier`), `t.*` or `s.t.*` expands to. Port of
/// [`ExpandColumnRefStar`]: `*` expands to the FROM items of the innermost level
/// ([`ExpandAllTables`]), and `t.*` and `s.t.*` to the item that the qualifier names in any
/// level ([`refnameNamespaceItem`], [`ExpandSingleTable`]). The columns of an item whose types
/// are not known as a whole have no types. `None` if the columns are not known, also if a level
/// has items whose names we don't know, which may be the one the qualifier names.
///
/// [`ExpandColumnRefStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1123
/// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
/// [`refnameNamespaceItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L134
/// [`ExpandSingleTable`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1401
fn expand_column_ref_star(r: &Resolver, qualifier: &[String]) -> Option<Vec<TypedColumn>> {
    if qualifier.is_empty() {
        let (columns, _) = r.levels.last()?.star_columns()?;
        return Some(columns);
    }
    if has_opaque_items(&r.levels) {
        return None;
    }
    let item = find_qualified_item(&r.levels, qualifier)?;
    Some(
        item.columns
            .as_ref()?
            .iter()
            .map(|column| TypedColumn {
                name: column.name.clone(),
                ty: column.ty.clone().filter(|_| item.types_known),
            })
            .collect(),
    )
}

/// The columns that `(arg).*` expands to, where `fields` are the subscripts and field
/// selections between `arg` and the `*`: the fields of its composite type. Port of
/// [`ExpandIndirectionStar`] and [`ExpandRowReference`]. `None` if the type or its fields are
/// not known, or if it isn't composite, which Postgres rejects.
///
/// [`ExpandIndirectionStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1444
/// [`ExpandRowReference`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1486
fn expand_indirection_star(
    r: &mut Resolver,
    arg: Option<&Node>,
    fields: &[Node],
) -> Option<Vec<TypedColumn>> {
    let ty = infer_indirection(r, arg, fields)?;
    row_columns(r, &ty).flatten()
}

/// Types the expressions of a select list or RETURNING list whose output is `output`, the
/// [`select_list_output`] of `targets`. Each is typed once, after the other clauses of the
/// query, in order. Typing them first, like Postgres, would only change the order of findings
/// and references.
pub(super) fn infer_select_list(r: &mut Resolver, targets: &[Node], output: &mut QueryOutput) {
    for (target, entry) in targets.iter().zip(&mut output.targets) {
        if entry.star.is_some() {
            continue;
        }
        let Some(NodeEnum::ResTarget(target)) = target.node.as_ref() else {
            continue;
        };
        let Some(value) = target.val.as_deref().and_then(|value| value.node.as_ref()) else {
            continue;
        };
        let ty = infer_expr(r, value);
        if let Some(column) = entry
            .columns
            .as_mut()
            .and_then(|columns| columns.first_mut())
        {
            column.ty = ty;
        }
    }
}

/// Records the columns that each `*`, `t.*` and `s.t.*` of a select list or RETURNING list
/// expands to, when references are collected and the columns are known. The FROM items of the
/// list must be the innermost level.
pub(super) fn record_star_references(r: &mut Resolver, output: &QueryOutput) {
    if !r.collects_references() {
        return;
    }
    for target in &output.targets {
        let (Some(Star::Columns(qualifier)), Some(columns)) = (&target.star, &target.columns)
        else {
            continue;
        };
        let columns = match qualifier.as_slice() {
            [] => r
                .levels
                .last()
                .and_then(|level| star_columns(level, columns)),
            _ => find_qualified_item(&r.levels, qualifier)
                .map(|item| item_columns(item, qualifier, columns)),
        };
        if let Some(columns) = columns {
            r.record(Reference::Star {
                location: target.location,
                span: r.star_span(target.location),
                columns,
            });
        }
    }
}

/// The columns of `*` in `level`, like [`ExpandAllTables`], which expands to `columns`, with the
/// item of each from [`Level::star`]. A column is qualified with the name of its FROM item when
/// there are several, and a column merged by `JOIN ... USING` or a `NATURAL` join is referenced
/// by its bare name. A column has no qualifier if no name references it with certainty.
///
/// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
fn star_columns(level: &Level, columns: &[TypedColumn]) -> Option<Vec<StarColumn>> {
    if let [item] = level.items.as_slice() {
        return Some(item_columns(item, &[], columns));
    }
    let entries = level.star()?;
    if entries.len() != columns.len() {
        return None;
    }
    let names: Vec<&str> = columns.iter().map(|column| column.name.as_str()).collect();
    let columns = entries
        .iter()
        .zip(&names)
        .map(|(entry, name)| {
            let qualifier = match entry {
                // The bare name must reference only the merged column.
                StarEntry::Merged(_) => {
                    (names.iter().filter(|other| *other == name).count() == 1).then(Vec::new)
                }
                // The qualifier must reference only this item, and the name only this column.
                StarEntry::Item { item, .. } => level.items.get(*item).and_then(|item| {
                    let item_name = item.name.as_ref()?;
                    let unique_item = level
                        .items
                        .iter()
                        .filter(|other| other.name.as_ref() == Some(item_name))
                        .count()
                        == 1;
                    (unique_item && has_unique_column(item, name)).then(|| vec![item_name.clone()])
                }),
            };
            StarColumn {
                qualifier,
                name: (*name).to_owned(),
            }
        })
        .collect();
    Some(columns)
}

/// The columns `columns` of `item`, qualified with `qualifier`. A column whose name another
/// column of the item has too has no qualifier: its name doesn't reference it.
fn item_columns(item: &Item, qualifier: &[String], columns: &[TypedColumn]) -> Vec<StarColumn> {
    columns
        .iter()
        .map(|column| StarColumn {
            qualifier: has_unique_column(item, &column.name).then(|| qualifier.to_vec()),
            name: column.name.clone(),
        })
        .collect()
}

/// Whether `item` has exactly one column named `name`.
fn has_unique_column(item: &Item, name: &str) -> bool {
    item.columns
        .iter()
        .flatten()
        .filter(|column| column.name == name)
        .count()
        == 1
}

/// Resolves a RETURNING clause of a statement on `target` against `level` and returns its
/// columns.
pub(super) fn resolve_returning_clause(
    r: &mut Resolver,
    clause: Option<&ReturningClause>,
    target: &Item,
    mut level: Level,
) -> Option<QueryOutput> {
    let (options, list) = clause.map_or((&[][..], &[][..]), |clause| {
        (clause.options.as_slice(), clause.exprs.as_slice())
    });
    add_old_and_new(r, options, target, &mut level);
    r.enter_level(level);
    resolve_list(r, list);
    let mut output = select_list_output(r, list);
    record_star_references(r, &output);
    infer_select_list(r, list, &mut output);
    r.exit_level();
    Some(output)
}

/// Resolves the `SET` list of UPDATE, `ON CONFLICT DO UPDATE`, or `MERGE ... UPDATE`: each
/// target column must exist, each value must be assignable to it, and generated columns can
/// only be set to `DEFAULT` ([`check_generated_update`]). Port of
/// [`transformUpdateTargetList`].
///
/// [`transformUpdateTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2546
pub(super) fn resolve_set_clause(r: &mut Resolver, target: &Item, assignments: &[Node]) {
    check_generated_update(r, target, assignments);
    for assignment in assignments {
        if let Some(NodeEnum::ResTarget(assignment)) = &assignment.node {
            check_target_column(r, target, &assignment.name, assignment.location);
            // Assignments to subscripts or fields, like `a[1]` or `c.f`, aren't checked.
            if let (true, Some(column_type), Some(expr)) = (
                assignment.indirection.is_empty(),
                target.type_of(&assignment.name).flatten(),
                assignment.val.as_deref().and_then(|v| v.node.as_ref()),
            ) {
                let expr_type = infer_expr(r, expr);
                check_assignment(
                    r,
                    &assignment.name,
                    column_type,
                    Some(expr),
                    expr_type,
                    assignment
                        .val
                        .as_deref()
                        .and_then(node_location)
                        .unwrap_or(assignment.location),
                );
            }
        }
        super::resolve_node(r, assignment);
    }
}

/// Reports a column of the target relation of INSERT or UPDATE that doesn't exist, like
/// [`checkInsertTargets`] and [`transformUpdateTargetList`].
///
/// [`checkInsertTargets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1018
/// [`transformUpdateTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2546
pub(super) fn check_target_column(r: &mut Resolver, target: &Item, column: &str, location: i32) {
    if target.has_column(column) == Some(false) {
        r.report(
            FindingKind::UnknownColumn {
                relation: target.name.clone(),
                column: column.to_owned(),
            },
            location,
        );
    }
}

/// Reports an UPDATE, `ON CONFLICT DO UPDATE` or `MERGE ... UPDATE` that sets a generated
/// column or a `GENERATED ALWAYS` identity column to anything but `DEFAULT`. Port of the checks
/// of [`rewriteTargetListIU`] for `CMD_UPDATE`, which no `OVERRIDING` clause affects.
/// [`RewriteQuery`] applies them to the UPDATE action of MERGE too.
///
/// Assignments to subscripts or fields of a column, like `g[1]`, are not checked.
///
/// [`rewriteTargetListIU`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L808
/// [`RewriteQuery`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L4162
fn check_generated_update(r: &mut Resolver, target: &Item, assignments: &[Node]) {
    if target.default_only_columns.is_empty() {
        return;
    }
    for assignment in assignments {
        let Some(NodeEnum::ResTarget(assignment)) = &assignment.node else {
            continue;
        };
        if !assignment.indirection.is_empty() {
            continue;
        }
        let Some(kind) = target.default_only(&assignment.name, false) else {
            continue;
        };
        let value = assignment
            .val
            .as_deref()
            .and_then(|value| value.node.as_ref());
        let is_default = match value {
            Some(NodeEnum::SetToDefault(_)) => Some(true),
            // `SET (a, g) = (1, DEFAULT)` and `SET (a, g) = (select ...)`.
            Some(NodeEnum::MultiAssignRef(multi)) => {
                match multi
                    .source
                    .as_deref()
                    .and_then(|source| source.node.as_ref())
                {
                    Some(NodeEnum::RowExpr(row)) => usize::try_from(multi.colno - 1)
                        .ok()
                        .and_then(|index| row.args.get(index))
                        .map(|value| matches!(value.node, Some(NodeEnum::SetToDefault(_)))),
                    Some(NodeEnum::SubLink(_)) => Some(false),
                    _ => None,
                }
            }
            Some(_) => Some(false),
            None => None,
        };
        if is_default == Some(false) {
            r.report(
                FindingKind::GeneratedColumnAssignment {
                    column: assignment.name.clone(),
                    insert: false,
                    identity: kind == DefaultOnly::IdentityAlways,
                },
                assignment.location,
            );
        }
    }
}

/// Since Postgres 18, `old` and `new` in a RETURNING list are the target row before and after
/// the change. They are only reachable by name. `RETURNING WITH (OLD AS o, NEW AS n)` renames
/// them; a name that is not given explicitly is only added if no visible item, in this or an
/// outer query, has that name. Port of
/// [`transformReturningClause`] and of [`addNSItemForReturning`]; an alias given twice, or one
/// that clashes with another item, is rejected by Postgres and not reported here.
///
/// [`transformReturningClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2661
/// [`addNSItemForReturning`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2621
fn add_old_and_new(r: &mut Resolver, options: &[Node], target: &Item, level: &mut Level) {
    if r.catalog.server_version_at_least(18) == Some(false) {
        return;
    }
    let explicit = |kind: ReturningOptionKind| {
        options
            .iter()
            .find_map(|option| match option.node.as_ref() {
                Some(NodeEnum::ReturningOption(option)) if option.option() == kind => Some(option),
                _ => None,
            })
    };
    let mut aliases = Vec::new();
    for (kind, default) in [
        (ReturningOptionKind::ReturningOptionOld, "old"),
        (ReturningOptionKind::ReturningOptionNew, "new"),
    ] {
        let name = match explicit(kind) {
            Some(option) => {
                // The location of an option is that of its `OLD` or `NEW`.
                r.record_alias_name(option.location, &option.value);
                option.value.as_str()
            }
            // `refnameNamespaceItem` searches the outer queries too.
            None if level.item(default).is_none() && find_item(&r.levels, default).is_none() => {
                default
            }
            None => continue,
        };
        let mut item = target.clone();
        item.name = Some(name.to_owned());
        item.schema = None;
        aliases.push(item);
    }
    level.qualified_only.extend(aliases);
}
