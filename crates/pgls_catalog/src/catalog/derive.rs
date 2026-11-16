//! Output columns of queries, for `CREATE TABLE AS`, `SELECT INTO` and views.

use super::Catalog;
use crate::lookup::ColumnInfo;
use crate::typing::{TypedColumn, resolve_unknown_outputs};

impl Catalog {
    /// The columns of a relation that a query with the `output` columns creates, or `None` when
    /// they can't be derived with certainty. Unknown literals become `text`.
    pub(super) fn query_columns(&self, output: Option<&[TypedColumn]>) -> Option<Vec<ColumnInfo>> {
        let columns = resolve_unknown_outputs(self, Some(output?.to_vec()))?;
        // Postgres rejects a relation with duplicate column names.
        if columns
            .iter()
            .enumerate()
            .any(|(i, column)| columns[..i].iter().any(|other| other.name == column.name))
        {
            return None;
        }
        Some(
            columns
                .into_iter()
                .map(|column| ColumnInfo::new(column.name, None, column.ty))
                .collect(),
        )
    }
}
