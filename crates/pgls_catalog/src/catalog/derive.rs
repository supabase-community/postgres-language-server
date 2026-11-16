//! Output columns of queries, for `CREATE TABLE AS`, `SELECT INTO` and views.

use pgls_query::protobuf;

use super::{Catalog, QueryColumnsHook};
use crate::lookup::ColumnInfo;

impl Catalog {
    /// The output columns of a query from `query_columns`, or `None` when they can't be derived
    /// with certainty.
    pub(super) fn query_columns(
        &self,
        query: &protobuf::Node,
        search_path: &[String],
        query_columns: QueryColumnsHook<'_>,
    ) -> Option<Vec<ColumnInfo>> {
        let columns = query_columns(query, self, search_path)?;
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
