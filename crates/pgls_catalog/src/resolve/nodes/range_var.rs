//! Relations and CTEs referenced by name.

use std::rc::Rc;

use pgls_query::protobuf::RangeVar;

use super::{Resolver, alias::apply_alias};
use crate::lookup::{Lookup, RelationKind};
use crate::resolve::{
    ColumnSource, FindingKind, Reference,
    scope::{Columns, Item, ItemSource},
};

/// A relation or CTE in FROM. A name without schema is a CTE first ([`transformFromClauseItem`]
/// and [`addRangeTableEntryForCTE`]).
///
/// [`transformFromClauseItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L1054
/// [`addRangeTableEntryForCTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L2364
pub(super) fn resolve_range_var(r: &mut Resolver, n: &RangeVar) -> Item {
    let alias = n.alias.as_ref();
    let is_unqualified = n.schemaname.is_empty() && n.catalogname.is_empty();
    if is_unqualified
        && let Some(cte) = r
            .ctes
            .iter()
            .rev()
            .find(|cte| cte.name == n.relname)
            .cloned()
    {
        if r.collects_references() {
            r.record(Reference::Cte {
                location: n.location,
                span: r.name_span(n.location),
                name: cte.name.clone(),
            });
            r.record_alias(n.location, alias);
        }
        let mut item = Item::named(
            Some(alias.map_or_else(|| cte.name.clone(), |alias| alias.aliasname.clone())),
            apply_alias(cte.columns.clone(), alias),
        );
        item.typed_columns = cte.typed_columns.clone().map(|mut columns| {
            if let Some(alias) = alias
                && let Some(names) = super::string::string_values(&alias.colnames)
            {
                for (column, name) in columns.iter_mut().zip(names) {
                    column.name = name;
                }
            }
            columns
        });
        if r.collects_references()
            && let Some(names) = &cte.columns
        {
            item.source = Some(Rc::new(ItemSource::derived(
                r,
                ColumnSource::Cte {
                    name: cte.name.clone(),
                },
                names,
                cte.typed_columns.as_deref(),
            )));
        }
        return item;
    }
    resolve_target_relation(r, n)
}

/// A relation that is never a CTE: the target of INSERT, UPDATE, and DELETE.
pub(super) fn resolve_target_relation(r: &mut Resolver, n: &RangeVar) -> Item {
    let alias = n.alias.as_ref();
    let relation = lookup_relation(r, n);
    // Like the relation, its alias only has a reference if the relation is known.
    if relation.schema.is_some() {
        r.record_alias(n.location, alias);
    }
    Item {
        name: Some(alias.map_or_else(|| n.relname.clone(), |alias| alias.aliasname.clone())),
        schema: relation.schema.filter(|_| alias.is_none()),
        columns: apply_alias(relation.columns, alias),
        typed_columns: relation.typed_columns.map(|mut columns| {
            if let Some(alias) = alias {
                for (column, name) in columns.iter_mut().zip(
                    crate::resolve::nodes::string::string_values(&alias.colnames)
                        .unwrap_or_default(),
                ) {
                    column.name = name;
                }
            }
            columns
        }),
        has_system_columns: relation.has_system_columns,
        source: relation.source,
    }
}

/// A relation found in the catalog. Everything is `None` if it was not found.
#[derive(Debug, Default)]
pub(super) struct ResolvedRelation {
    pub schema: Option<String>,
    pub columns: Columns,
    pub typed_columns: Option<Vec<crate::typing::TypedColumn>>,
    pub has_system_columns: bool,
    /// Only set when references are collected.
    pub source: Option<Rc<ItemSource>>,
}

/// Looks up a relation. Reports it if it doesn't exist. Port of [`RangeVarGetRelidExtended`],
/// which searches the schemas of the search path in order.
///
/// [`RangeVarGetRelidExtended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L441
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
            let mut source = None;
            if r.collects_references() {
                source = relation.columns.as_ref().map(|columns| {
                    Rc::new(ItemSource {
                        source: ColumnSource::Relation {
                            schema: relation.schema.clone(),
                            name: relation.name.clone(),
                        },
                        columns: columns.clone(),
                    })
                });
                r.record(Reference::Relation {
                    location: n.location,
                    span: r.name_span(n.location),
                    relation: relation.clone(),
                });
            }
            ResolvedRelation {
                schema: Some(relation.schema),
                columns: relation
                    .columns
                    .as_ref()
                    .map(|columns| columns.iter().map(|column| column.name.clone()).collect()),
                typed_columns: relation.columns.map(|columns| {
                    columns
                        .into_iter()
                        .map(|column| crate::typing::TypedColumn {
                            name: column.name,
                            ty: column.ty,
                        })
                        .collect()
                }),
                has_system_columns: relation.kind != RelationKind::View,
                source,
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
