use pgls_query::protobuf::CommonTableExpr;

use super::{Resolver, alias::rename_columns, resolve_node};
use crate::resolve::scope::Columns;

/// Resolves the query of a CTE and returns its columns, if known.
pub(super) fn resolve_common_table_expr(r: &mut Resolver, n: &CommonTableExpr) -> Columns {
    let columns = n
        .ctequery
        .as_deref()
        .and_then(|query| resolve_node(r, query));
    let columns = rename_columns(columns, &n.aliascolnames);
    // SEARCH and CYCLE add columns.
    if n.search_clause.is_some() || n.cycle_clause.is_some() {
        return None;
    }
    columns
}
