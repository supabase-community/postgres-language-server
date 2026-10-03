//! The names visible to an expression: FROM items grouped by query level, CTEs, and the
//! parameters of the SQL function being resolved.

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
}

impl Item {
    pub fn named(name: Option<String>, columns: Columns) -> Self {
        Self {
            name,
            schema: None,
            columns,
            typed_columns: None,
            has_system_columns: false,
        }
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

impl Level {
    pub fn with_item(item: Item) -> Self {
        Self {
            items: vec![item],
            ..Default::default()
        }
    }

    pub fn extend(&mut self, other: Level) {
        self.items.extend(other.items);
        self.merged_columns.extend(other.merged_columns);
        self.merged_typed_columns.extend(other.merged_typed_columns);
        self.has_natural_join |= other.has_natural_join;
        self.has_opaque_items |= other.has_opaque_items;
    }

    /// Typed columns of `SELECT *`, including columns merged by USING.
    pub fn star_typed_columns(&self) -> TypedColumns {
        if self.has_natural_join || self.items.is_empty() {
            return None;
        }
        let mut output = self.merged_typed_columns.clone();
        for item in &self.items {
            let columns = item.typed_columns.as_ref()?;
            output.extend(
                columns
                    .iter()
                    .filter(|column| !self.merged_columns.contains(&column.name))
                    .cloned(),
            );
        }
        Some(output)
    }

    /// The columns of `SELECT *`, if known.
    pub fn star_columns(&self) -> Columns {
        if self.has_natural_join || self.items.is_empty() {
            return None;
        }
        let mut columns = self.merged_columns.clone();
        for item in &self.items {
            columns.extend(
                item.columns
                    .clone()?
                    .into_iter()
                    .filter(|column| !self.merged_columns.contains(column)),
            );
        }
        Some(columns)
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
