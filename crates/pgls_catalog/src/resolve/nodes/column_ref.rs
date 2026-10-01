use pgls_query::{NodeEnum, protobuf::ColumnRef};

use super::{Resolver, string::string_values};
use crate::lookup::Lookup;
use crate::resolve::{
    FindingKind,
    scope::{ColumnLookup, find_column, find_item, has_opaque_items},
};

pub(super) fn resolve_column_ref(r: &mut Resolver, n: &ColumnRef) {
    // `*` and `t.*`
    let Some(last) = n.fields.last() else {
        return;
    };
    if matches!(last.node, Some(NodeEnum::AStar(_))) {
        return;
    }
    let Some(names) = string_values(&n.fields) else {
        return;
    };

    match names.as_slice() {
        [name] => resolve_unqualified_column(r, name, n.location),
        [qualifier, name] => resolve_qualified_column(r, qualifier, name, n.location),
        [schema, relation, name] => {
            // `schema.table.column`, if `schema.table` is an unaliased item.
            let item = r.levels.iter().rev().find_map(|level| {
                level.items.iter().find(|item| {
                    item.schema.as_deref() == Some(schema.as_str())
                        && item.name.as_deref() == Some(relation.as_str())
                })
            });
            if item.is_some_and(|item| item.has_column(name) == Some(false))
                && !may_be_function_call(r, name)
            {
                r.report(
                    FindingKind::UnknownColumn {
                        relation: Some(format!("{schema}.{relation}")),
                        column: name.clone(),
                    },
                    n.location,
                );
            }
        }
        _ => {}
    }
}

fn resolve_unqualified_column(r: &mut Resolver, name: &str, location: i32) {
    match find_column(&r.levels, name) {
        ColumnLookup::Found | ColumnLookup::Unknown => {}
        ColumnLookup::Ambiguous(candidates) => r.report(
            FindingKind::AmbiguousColumn {
                column: name.to_owned(),
                candidates,
            },
            location,
        ),
        ColumnLookup::NotFound => {
            // A whole-row reference to a FROM item, or a parameter of the SQL function.
            let is_item = find_item(&r.levels, name).is_some();
            if is_item || r.function_param(name).is_some() || has_opaque_items(&r.levels) {
                return;
            }
            r.report(
                FindingKind::UnknownColumn {
                    relation: None,
                    column: name.to_owned(),
                },
                location,
            );
        }
    }
}

fn resolve_qualified_column(r: &mut Resolver, qualifier: &str, name: &str, location: i32) {
    if let Some(item) = find_item(&r.levels, qualifier) {
        if item.has_column(name) == Some(false) && !may_be_function_call(r, name) {
            r.report(
                FindingKind::UnknownColumn {
                    relation: Some(qualifier.to_owned()),
                    column: name.to_owned(),
                },
                location,
            );
        }
        return;
    }

    // Postgres also reads `a.b` as field `b` of the composite column `a`, or as the function
    // call `b(a)`.
    if find_column(&r.levels, qualifier) != ColumnLookup::NotFound || has_opaque_items(&r.levels) {
        return;
    }

    if let Some(function) = r.function {
        // `function_name.parameter`
        if function.function_name == qualifier {
            return;
        }
        // A field of a composite parameter.
        if let Some(param) = r.function_param(qualifier) {
            if param.is_array {
                return;
            }
            let attributes = match r.catalog.type_(
                param.type_schema.as_deref(),
                &param.type_name,
                r.search_path,
            ) {
                Lookup::Found(type_info) => type_info.attributes,
                _ => None,
            };
            if let Some(attributes) = attributes {
                if !attributes.iter().any(|attribute| attribute.name == name)
                    && !may_be_function_call(r, name)
                {
                    r.report(
                        FindingKind::UnknownColumn {
                            relation: Some(qualifier.to_owned()),
                            column: name.to_owned(),
                        },
                        location,
                    );
                }
            }
            return;
        }
    }

    r.report(
        FindingKind::MissingFromClauseEntry {
            name: qualifier.to_owned(),
        },
        location,
    );
}

/// Postgres reads `t.name` as the function call `name(t)` when `t` has no column `name`
/// (`parse_expr.c: transformColumnRef`).
fn may_be_function_call(r: &Resolver, name: &str) -> bool {
    match r.catalog.functions(None, name, r.search_path) {
        Lookup::Found(overloads) => overloads
            .iter()
            .any(|overload| overload.min_args <= 1 && overload.max_args.is_none_or(|max| max >= 1)),
        Lookup::Missing => false,
        Lookup::Unknown => true,
    }
}
