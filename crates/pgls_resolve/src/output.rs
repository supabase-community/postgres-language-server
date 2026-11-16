//! The output columns of queries, with names and types together.

use pgls_text_size::TextRange;

use pgls_catalog::typing::{Type, TypedColumn};

/// The output columns of one entry of a select list or RETURNING list, or of one column of
/// VALUES.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetOutput {
    /// The location of the entry's value. For `*`, `t.*` and `s.t.*`, that of the column
    /// reference.
    pub location: i32,
    /// The location where the entry starts, with its alias: of the select list entry, or of the
    /// value of VALUES.
    pub(crate) entry_location: i32,
    /// The span of the whole entry, with its alias, like `coalesce(a, b) as x`. Only set for the
    /// [`Resolution::output`](super::Resolution::output) of a view, `CREATE TABLE AS` or
    /// `SELECT INTO` whose text is known.
    pub span: Option<TextRange>,
    /// Set if the entry is a `*` that expands to any number of columns.
    pub star: Option<Star>,
    /// The columns of the entry, in order: one for an expression, the ones it expands to for a
    /// `*` or `(expr).*`. `None` if they are not known. A column whose type is not known has no
    /// type.
    pub columns: Option<Vec<TypedColumn>>,
}

/// A select list entry that expands to any number of columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Star {
    /// `*`, `t.*` or `s.t.*`, with the names before the `*`: none for `*`.
    Columns(Vec<String>),
    /// `(expr).*`, the fields of a composite value.
    Fields,
}

/// The output columns of a query: one [`TargetOutput`] per entry of its select list or
/// RETURNING list, in order. A set operation has the entries of its left-most select, and VALUES
/// one per column.
///
/// What is known is known per entry: an entry whose columns are not known doesn't hide the names
/// and positions of the others, but the columns of the whole query are only known if those of
/// every entry are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryOutput {
    pub targets: Vec<TargetOutput>,
}

/// The entry of a [`QueryOutput`] that produces an output column.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OutputPosition<'a> {
    /// The index of the entry in [`QueryOutput::targets`].
    pub entry: usize,
    pub target: &'a TargetOutput,
}

impl QueryOutput {
    pub(crate) fn new(targets: Vec<TargetOutput>) -> Self {
        Self { targets }
    }

    /// The columns of all entries, if the columns of every entry are known.
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

    /// The names of the output columns, if the columns of every entry are known.
    pub fn names(&self) -> Option<Vec<String>> {
        Some(
            self.known_columns()?
                .map(|column| column.name.clone())
                .collect(),
        )
    }

    /// The output columns with their types, if the columns of every entry are known. A column
    /// whose type is not known has no type.
    pub fn columns(&self) -> Option<Vec<TypedColumn>> {
        Some(self.known_columns()?.cloned().collect())
    }

    /// The number of output columns, if the columns of every entry are known.
    pub fn width(&self) -> Option<usize> {
        Some(self.known_columns()?.count())
    }

    /// The first output column, if the entries up to it are known.
    pub(crate) fn first_column(&self) -> Option<&TypedColumn> {
        for target in &self.targets {
            if let Some(column) = target.columns.as_deref()?.first() {
                return Some(column);
            }
        }
        None
    }

    /// The entry that produces each output column, in order, as far as the entries are known:
    /// the columns after an entry whose columns are not known have no known position.
    pub(crate) fn positions(&self) -> Vec<OutputPosition<'_>> {
        let mut positions = Vec::new();
        for (entry, target) in self.targets.iter().enumerate() {
            let Some(columns) = &target.columns else {
                break;
            };
            positions.extend(columns.iter().map(|_| OutputPosition { entry, target }));
        }
        positions
    }

    /// The names of the entries that aren't a `*`, which `ORDER BY` and `GROUP BY` can reference
    /// by bare name ([`findTargetlistEntrySQL92`]).
    ///
    /// Postgres matches the columns that a `*` expands to as well. Those are columns of the FROM
    /// items, which the lookup finds with the same meaning, so they are left out. An ambiguous
    /// match, which Postgres rejects, is not reported.
    ///
    /// [`findTargetlistEntrySQL92`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2006
    pub(crate) fn output_names(&self) -> Vec<String> {
        self.targets
            .iter()
            .filter(|target| target.star.is_none())
            .filter_map(|target| target.columns.as_deref()?.first())
            .map(|column| column.name.clone())
            .collect()
    }

    /// Sets the types of the columns, in order.
    pub(crate) fn set_types(&mut self, types: impl IntoIterator<Item = Option<Type>>) {
        let columns = self
            .targets
            .iter_mut()
            .flat_map(|target| target.columns.iter_mut().flatten());
        for (column, ty) in columns.zip(types) {
            column.ty = ty;
        }
    }

    /// Forgets the types of all columns, but keeps them known: each is `None`.
    pub(crate) fn clear_types(&mut self) {
        for column in self
            .targets
            .iter_mut()
            .flat_map(|target| target.columns.iter_mut().flatten())
        {
            column.ty = None;
        }
    }
}
