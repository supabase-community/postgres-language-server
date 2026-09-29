//! FROM lists. Each FROM item returns the names it brings into scope.

use pgls_query::{Node, NodeEnum};

use super::{
    Resolver, join_expr::resolve_join_expr, range_function::resolve_range_function,
    range_subselect::resolve_range_subselect, range_var::resolve_range_var,
};
use crate::resolve::scope::{Item, Level};

/// Resolves a FROM list into the FROM items of a query level.
pub(super) fn resolve_from_clause(r: &mut Resolver, from: &[Node]) -> Level {
    let mut level = Level::default();
    for node in from {
        let item = resolve_from_item(r, node, &level);
        level.extend(item);
    }
    level
}

/// Resolves one FROM item. `preceding` holds the items before it, which LATERAL items and
/// functions can reference.
pub(super) fn resolve_from_item(r: &mut Resolver, node: &Node, preceding: &Level) -> Level {
    let item = match node.node.as_ref() {
        Some(NodeEnum::RangeVar(n)) => resolve_range_var(r, n),
        Some(NodeEnum::RangeSubselect(n)) => resolve_range_subselect(r, n, preceding),
        Some(NodeEnum::RangeFunction(n)) => resolve_range_function(r, n, preceding),
        Some(NodeEnum::JoinExpr(n)) => return resolve_join_expr(r, n, preceding),
        _ => {
            // TABLESAMPLE, XMLTABLE, JSON_TABLE, ...: we don't know what they expose.
            r.depends_on_file();
            return Level {
                items: vec![Item::named(None, None)],
                has_opaque_items: true,
                ..Default::default()
            };
        }
    };
    Level::with_item(item)
}
