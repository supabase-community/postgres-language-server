//! The columns that `*`, `t.*`, `s.t.*` and `(expr).*` expand to, and the order of the columns
//! of a join that merges columns with `USING` or `NATURAL`, which `*` over the join follows and
//! unqualified column references see. Port of [`ExpandColumnRefStar`], [`ExpandIndirectionStar`]
//! and the column lists of the `JoinExpr` case of [`transformFromClauseItem`].
//!
//! [`ExpandColumnRefStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1123
//! [`ExpandIndirectionStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1444
//! [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054

use pgls_query::Node;

use crate::{
    Reference, StarColumn,
    expr::infer_indirection,
    output::{QueryOutput, Star},
    resolver::Resolver,
    scope::{Item, Level, find_qualified_item, has_opaque_items},
    utils::row_columns,
};
use pgls_catalog::typing::{Type, TypedColumn};

/// The order of the columns that `*` expands to.
#[derive(Debug, Clone, Default)]
pub(super) enum StarOrder {
    /// The columns of each item, one item after the other, as without joins that merge columns.
    #[default]
    Items,
    /// The order of a level with a join that merges columns.
    Columns(Vec<StarEntry>),
    /// Not known, because a join's columns can't be merged with certainty.
    Unknown,
}

/// A column that `*` expands to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum StarEntry {
    /// A column that `JOIN ... USING` or a `NATURAL` join merges, by its name.
    Merged(String),
    /// The column at `column` in the columns of the item at `item` of the level.
    Item { item: usize, column: usize },
}

impl Level {
    /// The order of the columns of `*` once the items of `other` follow those of the level.
    pub(crate) fn star_order_with(&self, other: &Level) -> StarOrder {
        match (&self.star_order, &other.star_order) {
            (StarOrder::Items, StarOrder::Items) => StarOrder::Items,
            _ => match (self.star(), other.star()) {
                (Some(mut columns), Some(other_columns)) => {
                    let offset = self.items.len();
                    columns.extend(other_columns.into_iter().map(|column| match column {
                        StarEntry::Item { item, column } => StarEntry::Item {
                            item: item + offset,
                            column,
                        },
                        merged => merged,
                    }));
                    StarOrder::Columns(columns)
                }
                _ => StarOrder::Unknown,
            },
        }
    }

    /// The columns that `*` expands to, in order, like [`ExpandAllTables`]. A join expands to the
    /// columns it merges, then the other columns of its left side, then those of its right side
    /// ([`transformFromClauseItem`]). `None` if they are not known.
    ///
    /// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
    /// [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054
    pub(crate) fn star(&self) -> Option<Vec<StarEntry>> {
        match &self.star_order {
            StarOrder::Items => {
                let mut columns = Vec::new();
                for (item, entry) in self.items.iter().enumerate() {
                    let count = entry.columns.as_ref()?.len();
                    columns.extend((0..count).map(|column| StarEntry::Item { item, column }));
                }
                Some(columns)
            }
            StarOrder::Columns(columns) => Some(columns.clone()),
            StarOrder::Unknown => None,
        }
    }

    /// The name of a column of `*`.
    pub(crate) fn star_column_name<'a>(&'a self, column: &'a StarEntry) -> Option<&'a str> {
        match column {
            StarEntry::Merged(name) => Some(name),
            StarEntry::Item { item, column } => self
                .items
                .get(*item)?
                .columns
                .as_ref()?
                .get(*column)
                .map(|column| column.name.as_str()),
        }
    }

    /// The columns of `SELECT *`, including merged columns, and whether their types are known
    /// as a whole: if they are known for every item. The columns of an item whose types are not
    /// known as a whole have no types. The type of a merged column is unknown if nested joins
    /// merge the same name again. `None` if the columns are not known.
    pub(crate) fn star_columns(&self) -> Option<(Vec<TypedColumn>, bool)> {
        if self.items.is_empty() {
            return None;
        }
        let mut types_known = true;
        let columns = self
            .star()?
            .iter()
            .map(|column| match column {
                StarEntry::Merged(name) => Some(TypedColumn {
                    name: name.clone(),
                    ty: self.merged_column_type(name),
                }),
                StarEntry::Item { item, column } => {
                    let item = self.items.get(*item)?;
                    types_known &= item.types_known;
                    let column = item.columns.as_ref()?.get(*column)?;
                    Some(TypedColumn {
                        name: column.name.clone(),
                        ty: column.ty.clone().filter(|_| item.types_known),
                    })
                }
            })
            .collect::<Option<Vec<_>>>()?;
        Some((columns, types_known))
    }

    /// The type of a column that the joins of the level merge, unknown if nested joins merge
    /// the same name again.
    fn merged_column_type(&self, name: &str) -> Option<Type> {
        let mut merged = self
            .merged_columns
            .iter()
            .filter(|merged| merged.name == name);
        match (merged.next(), merged.next()) {
            (Some(merged), None) => merged.ty.clone(),
            _ => None,
        }
    }

    /// The type of a column of `*`, as [`Level::star_columns`] gives it, without building the
    /// others. `None` if it is not known.
    pub(crate) fn star_column_type(&self, column: &StarEntry) -> Option<Type> {
        match column {
            StarEntry::Merged(name) => self.merged_column_type(name),
            StarEntry::Item { item, column } => {
                let item = self.items.get(*item)?;
                let column = item.columns.as_ref()?.get(*column)?;
                column.ty.clone().filter(|_| item.types_known)
            }
        }
    }

    /// The columns a join merges, and the columns of `*` over it: the merged columns, then the
    /// other columns of the left side, then those of the right side. `NATURAL` merges the columns
    /// of the left side that the right side has too. Postgres rejects a merged column that either
    /// side doesn't have exactly once, so they are unknown then, as they are when the columns of
    /// a side are. Port of the column lists of the `JoinExpr` case of
    /// [`transformFromClauseItem`].
    ///
    /// [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054
    pub(crate) fn merge_join_columns(
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
        let (left_names, right_names) =
            (names(left, &left_columns)?, names(right, &right_columns)?);
        let merged = if natural {
            left_names
                .iter()
                .filter(|name| right_names.contains(name))
                .cloned()
                .collect()
        } else {
            using.to_vec()
        };
        let once =
            |names: &[String], name: &String| names.iter().filter(|n| *n == name).count() == 1;
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

    /// The columns of `*` in the level, like [`ExpandAllTables`], which expands to `columns`,
    /// with the item of each from [`Level::star`]. A column is qualified with the name of its
    /// FROM item when there are several, and a column merged by `JOIN ... USING` or a `NATURAL`
    /// join is referenced by its bare name. A column has no qualifier if no name references it
    /// with certainty.
    ///
    /// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
    fn star_references(&self, columns: &[TypedColumn]) -> Option<Vec<StarColumn>> {
        if let [item] = self.items.as_slice() {
            return Some(item.star_references(&[], columns));
        }
        let entries = self.star()?;
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
                    // The qualifier must reference only this item, and the name only this
                    // column.
                    StarEntry::Item { item, .. } => self.items.get(*item).and_then(|item| {
                        let item_name = item.name.as_ref()?;
                        let unique_item = self
                            .items
                            .iter()
                            .filter(|other| other.name.as_ref() == Some(item_name))
                            .count()
                            == 1;
                        (unique_item && item.has_unique_column(name))
                            .then(|| vec![item_name.clone()])
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
}

impl Item {
    /// The columns `columns` of the item, qualified with `qualifier`. A column whose name
    /// another column of the item has too has no qualifier: its name doesn't reference it.
    fn star_references(&self, qualifier: &[String], columns: &[TypedColumn]) -> Vec<StarColumn> {
        columns
            .iter()
            .map(|column| StarColumn {
                qualifier: self
                    .has_unique_column(&column.name)
                    .then(|| qualifier.to_vec()),
                name: column.name.clone(),
            })
            .collect()
    }

    /// Whether the item has exactly one column named `name`.
    fn has_unique_column(&self, name: &str) -> bool {
        self.columns
            .iter()
            .flatten()
            .filter(|column| column.name == name)
            .count()
            == 1
    }
}

impl Resolver<'_> {
    /// The columns that `*` (an empty `qualifier`), `t.*` or `s.t.*` expands to. Port of
    /// [`ExpandColumnRefStar`]: `*` expands to the FROM items of the innermost level
    /// ([`ExpandAllTables`]), and `t.*` and `s.t.*` to the item that the qualifier names in any
    /// level ([`refnameNamespaceItem`], [`ExpandSingleTable`]). The columns of an item whose
    /// types are not known as a whole have no types. `None` if the columns are not known, also
    /// if a level has items whose names we don't know, which may be the one the qualifier names.
    ///
    /// [`ExpandColumnRefStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1123
    /// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
    /// [`refnameNamespaceItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L134
    /// [`ExpandSingleTable`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1401
    pub(crate) fn expand_column_ref_star(&self, qualifier: &[String]) -> Option<Vec<TypedColumn>> {
        if qualifier.is_empty() {
            let (columns, _) = self.levels().last()?.star_columns()?;
            return Some(columns);
        }
        if has_opaque_items(self.levels()) {
            return None;
        }
        let item = find_qualified_item(self.levels(), qualifier)?;
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
    /// [`ExpandIndirectionStar`] and [`ExpandRowReference`]. `None` if the type or its fields
    /// are not known, or if it isn't composite, which Postgres rejects.
    ///
    /// [`ExpandIndirectionStar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1444
    /// [`ExpandRowReference`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1486
    pub(crate) fn expand_indirection_star(
        &mut self,
        arg: Option<&Node>,
        fields: &[Node],
    ) -> Option<Vec<TypedColumn>> {
        let ty = infer_indirection(self, arg, fields)?;
        row_columns(self, &ty).flatten()
    }

    /// Records the columns that each `*`, `t.*` and `s.t.*` of a select list or RETURNING list
    /// expands to, when references are collected and the columns are known. The FROM items of
    /// the list must be the innermost level.
    pub(crate) fn record_star_references(&mut self, output: &QueryOutput) {
        if !self.collects_references() {
            return;
        }
        for target in &output.targets {
            let (Some(Star::Columns(qualifier)), Some(columns)) = (&target.star, &target.columns)
            else {
                continue;
            };
            let columns = match qualifier.as_slice() {
                [] => self
                    .levels()
                    .last()
                    .and_then(|level| level.star_references(columns)),
                _ => find_qualified_item(self.levels(), qualifier)
                    .map(|item| item.star_references(qualifier, columns)),
            };
            if let Some(columns) = columns {
                self.record(Reference::Star {
                    location: target.location,
                    span: self.star_span(target.location),
                    columns,
                });
            }
        }
    }
}
