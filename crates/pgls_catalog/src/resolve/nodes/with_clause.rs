use pgls_query::{
    NodeEnum,
    protobuf::{CommonTableExpr, SetOperation, WithClause},
};

use super::{
    Resolver, alias::rename_typed_columns, common_table_expr::resolve_common_table_expr,
    select_stmt::resolve_select_stmt,
};
use crate::resolve::{
    resolve_unknown_outputs,
    scope::{Cte, TypedColumns},
};

/// Resolves the CTEs of a WITH clause and brings them into scope. The query that owns the
/// clause removes them again. Port of [`transformWithClause`].
///
/// [`transformWithClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L110
pub(super) fn resolve_with_clause(r: &mut Resolver, n: &WithClause) {
    let definitions: Vec<_> = n
        .ctes
        .iter()
        .filter_map(|node| match &node.node {
            Some(NodeEnum::CommonTableExpr(cte)) => Some(cte.as_ref()),
            _ => None,
        })
        .collect();

    // In WITH RECURSIVE, every CTE can reference every CTE of the list, including itself.
    if n.recursive {
        r.ctes.extend(definitions.iter().map(|cte| Cte {
            name: cte.ctename.clone(),
            columns: None,
            typed_columns: None,
        }));
    }

    for cte in definitions {
        let non_recursive = if n.recursive {
            non_recursive_term_columns(r, cte)
        } else {
            None
        };
        // The recursive term sees the columns of the non-recursive term.
        if let Some(Some(columns)) = &non_recursive
            && let Some(placeholder) = r.ctes.iter_mut().rev().find(|c| c.name == cte.ctename)
        {
            placeholder.columns = Some(columns.iter().map(|c| c.name.clone()).collect());
            placeholder.typed_columns = Some(columns.clone());
        }

        let columns = resolve_common_table_expr(r, cte);
        let typed_columns = if cte.search_clause.is_some() || cte.cycle_clause.is_some() {
            None
        } else {
            let output = rename_typed_columns(
                resolve_unknown_outputs(r.catalog, r.output.clone()),
                &cte.aliascolnames,
            );
            match non_recursive {
                Some(non_recursive) => recursive_cte_types(non_recursive, output),
                None => output,
            }
        };
        let resolved = Cte {
            name: cte.ctename.clone(),
            columns,
            typed_columns,
        };
        match r
            .ctes
            .iter_mut()
            .rev()
            .find(|visible| n.recursive && visible.name == cte.ctename)
        {
            Some(existing) => *existing = resolved,
            None => r.ctes.push(resolved),
        }
    }
}

/// The columns of the non-recursive term of a CTE in `WITH RECURSIVE`: `left` in
/// `left UNION [ALL] right`, named after the column list of the CTE, with unknown literals as
/// `text`. A recursive CTE has these columns, also in its recursive term `right`
/// ([`determineRecursiveColTypes`], [`analyzeCTETargetList`]).
///
/// `None` if the query is no UNION, so the CTE can't reference itself
/// ([`checkWellFormedRecursion`] rejects that) and has the columns of its query. `Some(None)` if
/// we don't know the columns of the left term, or the query has a WITH clause of its own that the
/// left term may reference.
///
/// [`determineRecursiveColTypes`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2387
/// [`analyzeCTETargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L571
/// [`checkWellFormedRecursion`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L915
fn non_recursive_term_columns(r: &mut Resolver, cte: &CommonTableExpr) -> Option<TypedColumns> {
    let query = match cte
        .ctequery
        .as_deref()
        .and_then(|query| query.node.as_ref())
    {
        Some(NodeEnum::SelectStmt(query)) if query.op() == SetOperation::SetopUnion => query,
        _ => return None,
    };
    if query.with_clause.is_some() {
        return Some(None);
    }
    let Some(left) = query.larg.as_deref() else {
        return Some(None);
    };
    resolve_select_stmt(r, left);
    Some(rename_typed_columns(
        resolve_unknown_outputs(r.catalog, r.output.clone()),
        &cte.aliascolnames,
    ))
}

/// The types of a CTE in `WITH RECURSIVE`. If it references itself, it has the types of its
/// non-recursive term, and Postgres rejects it unless the whole query has the same types
/// ([`analyzeCTE`]). If it doesn't, it has the types of the whole query. We don't tell the two
/// apart, so a column has a type only where both agree.
///
/// [`analyzeCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_cte.c#L385
fn recursive_cte_types(non_recursive: TypedColumns, output: TypedColumns) -> TypedColumns {
    let non_recursive = non_recursive?;
    let mut output = output?;
    if non_recursive.len() != output.len() {
        return None;
    }
    for (column, non_recursive) in output.iter_mut().zip(non_recursive) {
        if column.ty != non_recursive.ty {
            column.ty = None;
        }
    }
    Some(output)
}
