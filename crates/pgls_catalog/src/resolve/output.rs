//! The output columns of queries, with names and types together.

use crate::typing::{Type, TypedColumn};

/// The output columns of one entry of a select list or RETURNING list, or of one column of
/// VALUES.
#[derive(Debug, Clone)]
pub(super) struct TargetOutput {
    /// The location of the entry. For `*`, `t.*` and `s.t.*`, that of the column reference.
    pub location: i32,
    /// For `*`, `t.*` and `s.t.*`, the names before the `*`: empty for `*`.
    pub star: Option<Vec<String>>,
    /// The columns of the entry, in order: one for an expression, the ones it expands to for a
    /// `*`. `None` if they are not known.
    pub columns: Option<Vec<TypedColumn>>,
}

/// The output columns of a query: one [`TargetOutput`] per entry of its select list or
/// RETURNING list, in order. A set operation has the entries of its left-most select, and VALUES
/// one per column.
#[derive(Debug, Clone)]
pub(super) struct QueryOutput {
    pub targets: Vec<TargetOutput>,
    /// Whether the types of the columns are known. Not for a set operation whose arms don't have
    /// known columns of the same number.
    pub types_known: bool,
    /// Set if a `*` of a select list has types for only some of its columns. The types of the
    /// whole list are then `None`, once its expressions are typed.
    // PR 3: types are all-or-nothing too.
    pub star_types_missing: bool,
}

impl QueryOutput {
    pub fn new(targets: Vec<TargetOutput>) -> Self {
        Self {
            targets,
            types_known: true,
            star_types_missing: false,
        }
    }

    /// The columns of all entries, if the columns of every entry are known.
    // PR 3: the names are all-or-nothing: one unknown entry makes every column unknown, also to
    // the subqueries and CTEs that have this output.
    fn known_columns(&self) -> Option<impl Iterator<Item = &TypedColumn>> {
        if self.targets.iter().any(|target| target.columns.is_none()) {
            return None;
        }
        Some(
            self.targets
                .iter()
                .flat_map(|target| target.columns.iter().flatten()),
        )
    }

    /// The names of the output columns, if known.
    pub fn names(&self) -> Option<Vec<String>> {
        Some(self.known_columns()?.map(|column| column.name.clone()).collect())
    }

    /// The output columns with their types, if the columns and their types are known.
    pub fn columns(&self) -> Option<Vec<TypedColumn>> {
        Some(self.typed_columns()?.cloned().collect())
    }

    /// The first output column, if the columns and their types are known.
    pub fn first_column(&self) -> Option<&TypedColumn> {
        self.typed_columns()?.next()
    }

    fn typed_columns(&self) -> Option<impl Iterator<Item = &TypedColumn>> {
        if !self.types_known {
            return None;
        }
        self.known_columns()
    }

    /// The names of the entries that aren't a `*`, which `ORDER BY` and `GROUP BY` can reference
    /// by bare name ([`findTargetlistEntrySQL92`]).
    ///
    /// [`findTargetlistEntrySQL92`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2006
    // PR 3: Postgres matches the columns that a `*` expands to as well.
    pub fn output_names(&self) -> Vec<String> {
        self.targets
            .iter()
            .filter(|target| target.star.is_none())
            .filter_map(|target| target.columns.as_deref()?.first())
            .map(|column| column.name.clone())
            .collect()
    }

    /// Sets the types of the columns, in order.
    pub fn set_types(&mut self, types: impl IntoIterator<Item = Option<Type>>) {
        let columns = self
            .targets
            .iter_mut()
            .flat_map(|target| target.columns.iter_mut().flatten());
        for (column, ty) in columns.zip(types) {
            column.ty = ty;
        }
    }

    /// Forgets the types of all columns, but keeps them known: each is `None`.
    pub fn clear_types(&mut self) {
        for column in self
            .targets
            .iter_mut()
            .flat_map(|target| target.columns.iter_mut().flatten())
        {
            column.ty = None;
        }
    }
}
