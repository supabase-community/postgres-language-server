use pgls_query::protobuf::JoinExpr;

use super::{
    Resolver, alias::apply_alias, from_clause::resolve_from_item, resolve_node,
    string::string_values,
};
use crate::{
    resolve::scope::{Item, Level},
    typing::{Selection, TypedColumn, select_common_type},
};

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
    let merged = using_columns
        .iter()
        .map(|name| {
            let left_ty = left
                .items
                .iter()
                .find_map(|item| item.type_of(name).flatten());
            let right_ty = right
                .items
                .iter()
                .find_map(|item| item.type_of(name).flatten());
            let ty = match (left_ty, right_ty) {
                (Some(left), Some(right)) => match select_common_type(r.catalog, &[left, right]) {
                    Selection::Match(ty) => Some(ty),
                    _ => None,
                },
                _ => None,
            };
            TypedColumn {
                name: name.clone(),
                ty,
            }
        })
        .collect::<Vec<_>>();
    let mut joined = left;
    joined.extend(right);
    joined.merged_columns.extend(using_columns.iter().cloned());
    joined.merged_typed_columns.extend(merged);
    joined.has_natural_join |= n.is_natural;

    if let Some(quals) = n.quals.as_deref() {
        r.enter_level(joined);
        resolve_node(r, quals);
        if let Some(expr) = quals.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
        joined = r.exit_level();
    }

    if let Some(alias) = &n.alias {
        // Only the alias of the join is visible outside of it.
        let columns = if joined.has_natural_join || !joined.merged_columns.is_empty() {
            None
        } else {
            joined.star_columns()
        };
        let mut item = Item::named(
            Some(alias.aliasname.clone()),
            apply_alias(columns, Some(alias)),
        );
        if let Some(mut typed) = joined.star_typed_columns() {
            for (column, name) in typed.iter_mut().zip(&alias.colnames) {
                if let Some(pgls_query::NodeEnum::String(name)) = name.node.as_ref() {
                    column.name = name.sval.clone();
                }
            }
            item.typed_columns = Some(typed);
        }
        return Level {
            items: vec![item],
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
