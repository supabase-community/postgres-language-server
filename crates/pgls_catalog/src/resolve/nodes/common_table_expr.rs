use pgls_query::protobuf::{CommonTableExpr, Node};

use super::{Resolver, alias::rename_columns, resolve_node};
use crate::resolve::{scope::Columns, utils::common};
use crate::typing::{Type, TypedColumn};

/// Resolves the query of a CTE and returns its columns, if known, without the columns of
/// `SEARCH` and `CYCLE` ([`search_cycle_columns`]). Port of [`analyzeCTE`].
///
/// [`analyzeCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L243
pub(super) fn resolve_common_table_expr(r: &mut Resolver, n: &CommonTableExpr) -> Columns {
    let columns = n
        .ctequery
        .as_deref()
        .and_then(|query| resolve_node(r, query));
    rename_columns(columns, &n.aliascolnames)
}

/// The columns that `SEARCH` and `CYCLE` add after the columns of a CTE, like
/// [`addRangeTableEntryForCTE`]: the sequence column, a `record` for `BREADTH FIRST` and a
/// `record[]` for `DEPTH FIRST`, then the mark column and the path column, a `record[]`. The mark
/// has the common type of its two values, which are `true` and `false` unless the query sets them
/// ([`analyzeCTE`]).
///
/// [`addRangeTableEntryForCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L2439
/// [`analyzeCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L243
pub(super) fn search_cycle_columns(r: &mut Resolver, n: &CommonTableExpr) -> Vec<TypedColumn> {
    let record = r.catalog.builtin_type("record").map(Type::Named);
    let record_array = r.catalog.builtin_type("_record").map(Type::Named);
    let mut columns = Vec::new();
    if let Some(search) = &n.search_clause {
        columns.push(TypedColumn {
            name: search.search_seq_column.clone(),
            ty: if search.search_breadth_first {
                record
            } else {
                record_array.clone()
            },
        });
    }
    if let Some(cycle) = &n.cycle_clause {
        let mark_values: Option<Vec<Node>> = [&cycle.cycle_mark_value, &cycle.cycle_mark_default]
            .into_iter()
            .map(|value| value.as_deref().cloned())
            .collect();
        columns.push(TypedColumn {
            name: cycle.cycle_mark_column.clone(),
            ty: mark_values.and_then(|values| common(r, &values)),
        });
        columns.push(TypedColumn {
            name: cycle.cycle_path_column.clone(),
            ty: record_array,
        });
    }
    columns
}
