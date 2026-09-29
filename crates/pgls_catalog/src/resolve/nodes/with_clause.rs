use pgls_query::{NodeEnum, protobuf::WithClause};

use super::{Resolver, common_table_expr::resolve_common_table_expr};
use crate::resolve::scope::Cte;

/// Resolves the CTEs of a WITH clause and brings them into scope. The query that owns the
/// clause removes them again.
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
        }));
    }

    for cte in definitions {
        let resolved = Cte {
            name: cte.ctename.clone(),
            columns: resolve_common_table_expr(r, cte),
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
