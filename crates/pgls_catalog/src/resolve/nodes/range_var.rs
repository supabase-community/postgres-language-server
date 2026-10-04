//! Relations and CTEs referenced by name.

use pgls_query::protobuf::RangeVar;

use super::{Resolver, alias::apply_alias};
use crate::lookup::{Lookup, RelationKind};
use crate::resolve::{
    FindingKind,
    scope::{Columns, Item},
};

/// A relation or CTE in FROM.
pub(super) fn resolve_range_var(r: &mut Resolver, n: &RangeVar) -> Item {
    let alias = n.alias.as_ref();
    let is_unqualified = n.schemaname.is_empty() && n.catalogname.is_empty();
    if is_unqualified && let Some(cte) = r.ctes.iter().rev().find(|cte| cte.name == n.relname) {
        return Item::named(
            Some(alias.map_or_else(|| cte.name.clone(), |alias| alias.aliasname.clone())),
            apply_alias(cte.columns.clone(), alias),
        );
    }
    resolve_target_relation(r, n)
}

/// A relation that is never a CTE: the target of INSERT, UPDATE, and DELETE.
pub(super) fn resolve_target_relation(r: &mut Resolver, n: &RangeVar) -> Item {
    let alias = n.alias.as_ref();
    let relation = lookup_relation(r, n);
    Item {
        name: Some(alias.map_or_else(|| n.relname.clone(), |alias| alias.aliasname.clone())),
        schema: relation.schema.filter(|_| alias.is_none()),
        columns: apply_alias(relation.columns, alias),
        has_system_columns: relation.has_system_columns,
    }
}

/// A relation found in the catalog. Everything is `None` if it was not found.
#[derive(Debug, Default)]
pub(super) struct ResolvedRelation {
    pub schema: Option<String>,
    pub columns: Columns,
    pub has_system_columns: bool,
}

/// Looks up a relation. Reports it if it doesn't exist.
pub(super) fn lookup_relation(r: &mut Resolver, n: &RangeVar) -> ResolvedRelation {
    let unknown = ResolvedRelation::default();
    if !n.catalogname.is_empty() {
        r.depends_on_file();
        return unknown;
    }
    let schema = (!n.schemaname.is_empty()).then_some(n.schemaname.as_str());
    if let Some(schema) = schema
        && r.check_schema(schema, n.location)
    {
        return unknown;
    }

    match r.catalog.relation(schema, &n.relname, r.search_path) {
        Lookup::Found(relation) => {
            r.uses(relation.origin);
            ResolvedRelation {
                schema: Some(relation.schema),
                columns: relation
                    .columns
                    .map(|columns| columns.into_iter().map(|column| column.name).collect()),
                has_system_columns: relation.kind != RelationKind::View,
            }
        }
        Lookup::Missing => {
            r.report(
                FindingKind::UnknownRelation {
                    schema: schema.map(str::to_owned),
                    name: n.relname.clone(),
                },
                n.location,
            );
            unknown
        }
        Lookup::Unknown => {
            r.depends_on_file();
            unknown
        }
    }
}
