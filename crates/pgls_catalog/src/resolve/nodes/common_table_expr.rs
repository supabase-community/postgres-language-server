use pgls_query::protobuf::CommonTableExpr;

use super::{Resolver, alias::rename_columns, resolve_node};
use crate::resolve::scope::Columns;

/// Resolves the query of a CTE and returns its columns, if known. Port of [`analyzeCTE`].
///
/// `SEARCH` adds the sequence column after the CTE's columns, and `CYCLE` the mark and path
/// columns, like [`addRangeTableEntryForCTE`].
///
/// [`analyzeCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L243
/// [`addRangeTableEntryForCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L2439
pub(super) fn resolve_common_table_expr(r: &mut Resolver, n: &CommonTableExpr) -> Columns {
    let columns = n
        .ctequery
        .as_deref()
        .and_then(|query| resolve_node(r, query));
    let mut columns = rename_columns(columns, &n.aliascolnames)?;
    if let Some(search) = &n.search_clause {
        columns.push(search.search_seq_column.clone());
    }
    if let Some(cycle) = &n.cycle_clause {
        columns.push(cycle.cycle_mark_column.clone());
        columns.push(cycle.cycle_path_column.clone());
    }
    Some(columns)
}
