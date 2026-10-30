//! The names visible to an expression: FROM items grouped by query level, CTEs, and the
//! parameters of the SQL function being resolved.

use std::rc::Rc;

use super::{ColumnSource, resolver::Resolver};
use crate::ColumnInfo;
use crate::typing::{Type, TypedColumn};

/// Columns of an item in scope. `None` when they are not known.
pub(super) type Columns = Option<Vec<String>>;
pub(super) type TypedColumns = Option<Vec<TypedColumn>>;

/// A common table expression.
#[derive(Debug, Clone)]
pub(super) struct Cte {
    pub name: String,
    pub columns: Columns,
    pub typed_columns: TypedColumns,
}

/// Something in a FROM clause that columns can come from.
#[derive(Debug, Clone)]
pub(super) struct Item {
    /// The name the item is referenced by: its alias, or the name of the relation or function.
    /// `None` for subqueries without an alias.
    pub name: Option<String>,
    /// The schema of an unaliased relation, for `schema.table.column`.
    pub schema: Option<String>,
    pub columns: Columns,
    pub typed_columns: TypedColumns,
    /// Whether the item has system columns (`ctid`, `xmin`, ...).
    pub has_system_columns: bool,
    /// What the columns come from. Only set when references are collected.
    pub source: Option<Rc<ItemSource>>,
}

/// What the columns of a FROM item come from.
#[derive(Debug)]
pub(super) struct ItemSource {
    pub source: ColumnSource,
    /// The columns as the source defines them, in the order of [`Item::columns`].
    pub columns: Vec<ColumnInfo>,
}

impl ItemSource {
    /// The source of a CTE or subquery with these output columns.
    pub fn derived(
        r: &Resolver,
        source: ColumnSource,
        names: &[String],
        typed: Option<&[TypedColumn]>,
    ) -> Self {
        let typed = typed.filter(|typed| typed.len() == names.len());
        let columns = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let ty = typed.and_then(|typed| typed[i].ty.clone());
                ColumnInfo {
                    name: name.clone(),
                    type_name: match &ty {
                        Some(Type::Named(id)) => {
                            r.catalog.type_by_id(id).found().map(|info| info.name)
                        }
                        _ => None,
                    },
                    ty,
                }
            })
            .collect();
        Self { source, columns }
    }
}

impl Item {
    pub fn named(name: Option<String>, columns: Columns) -> Self {
        Self {
            name,
            schema: None,
            columns,
            typed_columns: None,
            has_system_columns: false,
            source: None,
        }
    }

    /// Where the column with this name comes from, if the item has exactly one such column and
    /// its source is known.
    pub fn column_source(&self, column: &str) -> Option<(ColumnSource, ColumnInfo)> {
        let source = self.source.as_deref()?;
        let mut positions = self
            .columns
            .as_ref()?
            .iter()
            .enumerate()
            .filter(|(_, name)| *name == column)
            .map(|(position, _)| position);
        let position = positions.next()?;
        if positions.next().is_some() {
            return None;
        }
        let info = source.columns.get(position)?;
        Some((source.source.clone(), info.clone()))
    }

    /// Whether the item certainly has the column. `None` if its columns are not known.
    pub fn type_of(&self, column: &str) -> Option<Option<Type>> {
        let columns = self.typed_columns.as_ref()?;
        columns
            .iter()
            .find(|c| c.name == column)
            .map(|c| c.ty.clone())
    }

    pub fn has_column(&self, column: &str) -> Option<bool> {
        let columns = self.columns.as_ref()?;
        Some(
            columns.iter().any(|c| c == column)
                || (self.has_system_columns && is_system_column(column)),
        )
    }
}

/// The FROM items of one query level.
#[derive(Debug, Clone, Default)]
pub(super) struct Level {
    pub items: Vec<Item>,
    /// The order of the columns of `*`.
    pub star_order: StarOrder,
    /// Columns merged by `JOIN ... USING`. An unqualified reference to them is not ambiguous.
    pub merged_columns: Vec<String>,
    /// Types of columns merged by USING, in USING-clause order.
    pub merged_typed_columns: Vec<TypedColumn>,
    /// Set if the level contains a `NATURAL` join, whose merged columns we don't compute.
    pub has_natural_join: bool,
    /// Set if the level contains items we don't model (`XMLTABLE`, `TABLESAMPLE`, ...), whose
    /// names we don't know.
    pub has_opaque_items: bool,
    /// The output column names of the select list, which `ORDER BY` and `GROUP BY` can
    /// reference by bare name.
    pub output_names: Vec<String>,
    /// Set while resolving the clauses that can reference `output_names`.
    pub output_names_visible: bool,
    /// Items only reachable by name, whose columns unqualified references and `*` don't see:
    /// `old` and `new` in a RETURNING list.
    pub qualified_only: Vec<Item>,
}

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
    pub fn with_item(item: Item) -> Self {
        Self {
            items: vec![item],
            ..Default::default()
        }
    }

    pub fn extend(&mut self, other: Level) {
        self.star_order = match (&self.star_order, &other.star_order) {
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
        };
        self.items.extend(other.items);
        self.merged_columns.extend(other.merged_columns);
        self.merged_typed_columns.extend(other.merged_typed_columns);
        self.has_natural_join |= other.has_natural_join;
        self.has_opaque_items |= other.has_opaque_items;
    }

    /// The columns that `*` expands to, in order, like [`ExpandAllTables`]. A join expands to the
    /// columns it merges, then the other columns of its left side, then those of its right side
    /// ([`transformFromClauseItem`]). `None` if they are not known.
    ///
    /// [`ExpandAllTables`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1296
    /// [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054
    pub fn star(&self) -> Option<Vec<StarEntry>> {
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
    pub fn star_column_name<'a>(&'a self, column: &'a StarEntry) -> Option<&'a str> {
        match column {
            StarEntry::Merged(name) => Some(name),
            StarEntry::Item { item, column } => self
                .items
                .get(*item)?
                .columns
                .as_ref()?
                .get(*column)
                .map(String::as_str),
        }
    }

    /// Typed columns of `SELECT *`, including merged columns. The type of a merged column is
    /// unknown if a `NATURAL` join merges it, or if nested joins merge the same name again.
    pub fn star_typed_columns(&self) -> TypedColumns {
        if self.items.is_empty() {
            return None;
        }
        self.star()?
            .iter()
            .map(|column| match column {
                StarEntry::Merged(name) => {
                    let mut merged = self
                        .merged_typed_columns
                        .iter()
                        .filter(|typed| typed.name == *name);
                    let ty = match (merged.next(), merged.next()) {
                        (Some(typed), None) => typed.ty.clone(),
                        _ => None,
                    };
                    Some(TypedColumn {
                        name: name.clone(),
                        ty,
                    })
                }
                StarEntry::Item { item, column } => self
                    .items
                    .get(*item)?
                    .typed_columns
                    .as_ref()?
                    .get(*column)
                    .cloned(),
            })
            .collect()
    }

    /// The columns of `SELECT *`, if known.
    pub fn star_columns(&self) -> Columns {
        if self.items.is_empty() {
            return None;
        }
        self.star()?
            .iter()
            .map(|column| self.star_column_name(column).map(str::to_owned))
            .collect()
    }

    pub fn item(&self, name: &str) -> Option<&Item> {
        self.items
            .iter()
            .chain(&self.qualified_only)
            .find(|item| item.name.as_deref() == Some(name))
    }
}

/// The result of looking up an unqualified column.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ColumnLookup {
    Found,
    /// Matches columns of several items of the same query level.
    Ambiguous(Vec<String>),
    /// Can't be decided, because some columns in scope are not known.
    Unknown,
    NotFound,
}

/// Looks up an unqualified column in the query levels, innermost first.
pub(super) fn find_column_type(levels: &[Level], column: &str) -> Option<Type> {
    for (depth, level) in levels.iter().rev().enumerate() {
        if depth == 0
            && level.output_names_visible
            && level.output_names.iter().any(|n| n == column)
        {
            return None;
        }
        if level.merged_columns.iter().any(|c| c == column) {
            return level
                .merged_typed_columns
                .iter()
                .find(|c| c.name == column)
                .and_then(|c| c.ty.clone());
        }
        let matches = level
            .items
            .iter()
            .filter(|item| item.has_column(column) == Some(true))
            .collect::<Vec<_>>();
        let uncertain = level.items.iter().any(|item| item.columns.is_none());
        if !matches.is_empty() {
            if matches.len() != 1
                || uncertain
                || level.merged_columns.iter().any(|c| c == column)
                || level.has_natural_join
                || is_system_column(column)
            {
                return None;
            }
            return matches[0].type_of(column).flatten();
        }
        if uncertain {
            return None;
        }
    }
    None
}

pub(super) fn find_column(levels: &[Level], column: &str) -> ColumnLookup {
    for (depth, level) in levels.iter().rev().enumerate() {
        if depth == 0
            && level.output_names_visible
            && level.output_names.iter().any(|name| name == column)
        {
            return ColumnLookup::Found;
        }

        let mut matches = Vec::new();
        let mut unknown = false;
        for item in &level.items {
            match item.has_column(column) {
                Some(true) => matches.push(item.name.clone().unwrap_or_default()),
                Some(false) => {}
                None => unknown = true,
            }
        }

        let merged = level.merged_columns.iter().any(|c| c == column);
        match matches.len() {
            0 if unknown => return ColumnLookup::Unknown,
            0 => {}
            1 => return ColumnLookup::Found,
            _ if unknown || merged || level.has_natural_join || is_system_column(column) => {
                return ColumnLookup::Found;
            }
            _ => return ColumnLookup::Ambiguous(matches),
        }
    }
    ColumnLookup::NotFound
}

/// The item that an unqualified column certainly comes from, innermost level first. `None` if
/// it is not known with certainty, or if it is an output column or a column merged by a join.
pub(super) fn find_column_item<'a>(levels: &'a [Level], column: &str) -> Option<&'a Item> {
    for (depth, level) in levels.iter().rev().enumerate() {
        if depth == 0
            && level.output_names_visible
            && level.output_names.iter().any(|name| name == column)
        {
            return None;
        }
        if level.merged_columns.iter().any(|c| c == column) {
            return None;
        }
        let mut found = None;
        for item in &level.items {
            match item.has_column(column) {
                Some(true) if found.is_none() => found = Some(item),
                Some(true) | None => return None,
                Some(false) => {}
            }
        }
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Finds the item with this name, innermost level first.
pub(super) fn find_item<'a>(levels: &'a [Level], name: &str) -> Option<&'a Item> {
    levels.iter().rev().find_map(|level| level.item(name))
}

/// Whether some level contains items whose names we don't know.
pub(super) fn has_opaque_items(levels: &[Level]) -> bool {
    levels.iter().any(|level| level.has_opaque_items)
}

pub(super) fn is_system_column(column: &str) -> bool {
    matches!(
        column,
        "ctid" | "xmin" | "xmax" | "cmin" | "cmax" | "tableoid"
    )
}
