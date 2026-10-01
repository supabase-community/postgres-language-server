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
        let columns = crate::resolve::query_output_columns(query, self, search_path)?;
        Some(
            columns
                .into_iter()
                .map(|name| ColumnInfo {
                    name,
                    type_name: None,

                    ty: None,
                })
                .collect(),
        )
    }
}
