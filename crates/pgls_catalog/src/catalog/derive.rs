//! Output columns of queries, for `CREATE TABLE AS`, `SELECT INTO` and views.

use pgls_query::protobuf;

use super::Catalog;
use crate::lookup::ColumnInfo;

impl Catalog {
    /// The output columns of a query, or `None` when they can't be derived with certainty.
    pub(super) fn query_columns(
        &self,
        query: &protobuf::Node,
        search_path: &[String],
    ) -> Option<Vec<ColumnInfo>> {
        let columns = crate::resolve::query_output_types(query, self, search_path)?;
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
                .map(|column| ColumnInfo {
                    name: column.name,
                    type_name: None,
                    ty: column.ty,
                })
                .collect(),
        )
    }
}
