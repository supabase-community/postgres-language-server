use pgls_query::protobuf::JoinExpr;

use super::{
    Resolver, alias::apply_alias, from_clause::resolve_from_item, resolve_node,
    string::string_values,
};
use crate::{
    resolve::scope::{Item, Level, StarEntry, StarOrder},
    typing::{Selection, TypedColumn, select_common_type},
};

/// A join. Its ON clause sees both sides of the join, but not the FROM items before it, and
/// `USING` merges the columns of both sides into their common type. `NATURAL` is `USING` with the
/// columns of the left side that the right side has too ([`transformFromClauseItem`] builds that
/// list in its [`isNatural`] case). Port of the `JoinExpr` case of [`transformFromClauseItem`],
/// with [`transformJoinUsingClause`], [`buildMergedJoinVar`] and [`transformJoinOnClause`].
///
/// [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054
/// [`isNatural`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1240
/// [`transformJoinUsingClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L306
/// [`buildMergedJoinVar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1666
/// [`transformJoinOnClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L365
pub(super) fn resolve_join_expr(r: &mut Resolver, n: &JoinExpr, preceding: &Level) -> Level {
    let (Some(left), Some(right)) = (n.larg.as_deref(), n.rarg.as_deref()) else {
        r.depends_on_file();
        return Level::default();
    };

    let left = resolve_from_item(r, left, preceding);
    // LATERAL items on the right can reference the left side.
    let mut preceding_right = preceding.clone();
    preceding_right.extend(left.clone());
    let right = resolve_from_item(r, right, &preceding_right);

    let using_columns = string_values(&n.using_clause).unwrap_or_default();
    let merge = (n.is_natural || !using_columns.is_empty())
        .then(|| merge_columns(&left, &right, n.is_natural, &using_columns));
    // `NATURAL` merges the columns that both sides have, like `USING` with them. They are only
    // known if all columns of both sides are known.
    let merged_columns = if n.is_natural {
        merge
            .as_ref()
            .and_then(|merge| merge.as_ref().map(|(names, _)| names.clone()))
    } else {
        Some(using_columns.clone())
    };
    let star_order = merge.map(|merge| match merge {
        Some((_, columns)) => StarOrder::Columns(columns),
        None => StarOrder::Unknown,
    });
    let merged = merged_columns
        .iter()
        .flatten()
        .map(|name| {
            let left_ty = left
                .items
                .iter()
                .find_map(|item| item.type_of(name).flatten());
            let right_ty = right
                .items
                .iter()
                .find_map(|item| item.type_of(name).flatten());
            let ty = match (left_ty, right_ty) {
                (Some(left), Some(right)) => match select_common_type(r.catalog, &[left, right]) {
                    Selection::Match(ty) => Some(ty),
                    _ => None,
                },
                _ => None,
            };
            TypedColumn {
                name: name.clone(),
                ty,
            }
        })
        .collect::<Vec<_>>();
    let mut joined = left;
    joined.extend(right);
    joined.has_natural_join |= merged_columns.is_none();
    joined
        .merged_columns
        .extend(merged_columns.into_iter().flatten());
    joined.merged_typed_columns.extend(merged);
    if let Some(star_order) = star_order {
        joined.star_order = star_order;
    }

    if let Some(quals) = n.quals.as_deref() {
        r.enter_level(joined);
        resolve_node(r, quals);
        if let Some(expr) = quals.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
        joined = r.exit_level();
    }

    if let Some(alias) = &n.alias {
        // Only the alias of the join is visible outside of it.
        let mut item = Item::named(
            Some(alias.aliasname.clone()),
            apply_alias(joined.star_columns(), Some(alias)),
        );
        if let Some(mut typed) = joined.star_typed_columns() {
            for (column, name) in typed.iter_mut().zip(&alias.colnames) {
                if let Some(pgls_query::NodeEnum::String(name)) = name.node.as_ref() {
                    column.name = name.sval.clone();
                }
            }
            item.typed_columns = Some(typed);
        }
        return Level {
            items: vec![item],
            has_opaque_items: joined.has_opaque_items,
            ..Default::default()
        };
    }

    if let Some(alias) = &n.join_using_alias {
        joined.items.push(Item::named(
            Some(alias.aliasname.clone()),
            Some(using_columns),
        ));
    }
    joined
}

/// The columns a join merges, and the columns of `*` over it: the merged columns, then the other
/// columns of the left side, then those of the right side. `NATURAL` merges the columns of the
/// left side that the right side has too. Postgres rejects a merged column that either side
/// doesn't have exactly once, so they are unknown then, as they are when the columns of a side
/// are. Port of the column lists of the `JoinExpr` case of [`transformFromClauseItem`].
///
/// [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054
fn merge_columns(
    left: &Level,
    right: &Level,
    natural: bool,
    using: &[String],
) -> Option<(Vec<String>, Vec<StarEntry>)> {
    let (left_columns, right_columns) = (left.star()?, right.star()?);
    let names = |level: &Level, columns: &[StarEntry]| {
        columns
            .iter()
            .map(|column| level.star_column_name(column).map(str::to_owned))
            .collect::<Option<Vec<_>>>()
    };
    let (left_names, right_names) = (names(left, &left_columns)?, names(right, &right_columns)?);
    let merged = if natural {
        left_names
            .iter()
            .filter(|name| right_names.contains(name))
            .cloned()
            .collect()
    } else {
        using.to_vec()
    };
    let once = |names: &[String], name: &String| names.iter().filter(|n| *n == name).count() == 1;
    if !merged
        .iter()
        .all(|name| once(&left_names, name) && once(&right_names, name) && once(&merged, name))
    {
        return None;
    }

    let offset = left.items.len();
    let mut columns: Vec<StarEntry> = merged.iter().cloned().map(StarEntry::Merged).collect();
    columns.extend(
        left_columns
            .into_iter()
            .zip(&left_names)
            .filter(|(_, name)| !merged.contains(name))
            .map(|(column, _)| column),
    );
    columns.extend(
        right_columns
            .into_iter()
            .zip(&right_names)
            .filter(|(_, name)| !merged.contains(name))
            .map(|(column, _)| match column {
                StarEntry::Item { item, column } => StarEntry::Item {
                    item: item + offset,
                    column,
                },
                merged => merged,
            }),
    );
    Some((merged, columns))
}
