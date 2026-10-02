use pgls_query::protobuf::JoinExpr;

use super::{
    Resolver, alias::apply_alias, from_clause::resolve_from_item, resolve_node,
    string::string_values,
};
use crate::resolve::scope::{Item, Level};

/// A join. Its ON clause sees both sides of the join, but not the FROM items before it.
pub(super) fn resolve_join_expr(r: &mut Resolver, n: &JoinExpr, preceding: &Level) -> Level {
    let (Some(left), Some(right)) = (n.larg.as_deref(), n.rarg.as_deref()) else {
        r.depends_on_file();
        return Level::default();
    };

    let left = resolve_from_item(r, left, preceding);
    // LATERAL items on the right can reference the left side.
    let mut preceding_right = preceding.clone();
    preceding_right.extend(left.clone());
    let right = resolve_from_item(r, right, &preceding_right);

    let using_columns = string_values(&n.using_clause).unwrap_or_default();
    let mut joined = left;
    joined.extend(right);
    joined.merged_columns.extend(using_columns.iter().cloned());
    joined.has_natural_join |= n.is_natural;

    if let Some(quals) = n.quals.as_deref() {
        r.enter_level(joined);
        resolve_node(r, quals);
        joined = r.exit_level();
    }

    if let Some(alias) = &n.alias {
        // Only the alias of the join is visible outside of it.
        let columns = if joined.has_natural_join || !joined.merged_columns.is_empty() {
            None
        } else {
            joined.star_columns()
        };
        return Level {
            items: vec![Item::named(
                Some(alias.aliasname.clone()),
                apply_alias(columns, Some(alias)),
            )],
            has_opaque_items: joined.has_opaque_items,
            ..Default::default()
        };
    }

    if let Some(alias) = &n.join_using_alias {
        joined.items.push(Item::named(
            Some(alias.aliasname.clone()),
            Some(using_columns),
        ));
    }
    joined
}
