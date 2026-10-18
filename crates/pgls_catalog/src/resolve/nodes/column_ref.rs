use pgls_query::{NodeEnum, protobuf::ColumnRef};

use super::{Resolver, string::string_values};
use crate::lookup::Lookup;
use crate::resolve::{
    FindingKind, Reference,
    scope::{ColumnLookup, Item, find_column, find_column_item, find_item, has_opaque_items},
};

/// Reports a column reference that matches no column, or columns of several FROM items, and a
/// qualifier that matches no FROM item. Port of the lookups of [`transformColumnRef`]:
/// [`colNameToVar`] for unqualified names, [`refnameNamespaceItem`] and [`scanNSItemForColumn`]
/// for qualified ones, with the errors of [`errorMissingColumn`] and [`errorMissingRTE`].
///
/// [`transformColumnRef`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L509
/// [`colNameToVar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L930
/// [`refnameNamespaceItem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L134
/// [`scanNSItemForColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L720
/// [`errorMissingColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L3803
/// [`errorMissingRTE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L3732
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
            let reference = column_reference(r, item, name, n.location);
            let missing = item.is_some_and(|item| item.has_column(name) == Some(false));
            record(r, reference);
            if missing && !may_be_function_call(r, name) {
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
        ColumnLookup::Found => {
            let reference = column_reference(r, find_column_item(&r.levels, name), name, location);
            record(r, reference);
        }
        ColumnLookup::Unknown => {}
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
        let reference = column_reference(r, Some(item), name, location);
        let missing = item.has_column(name) == Some(false);
        record(r, reference);
        if missing && !may_be_function_call(r, name) {
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
            if let Some(attributes) = attributes
                && !attributes.iter().any(|attribute| attribute.name == name)
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

/// The reference to the column `name` of `item`, if references are collected and the source of
/// the column is known.
fn column_reference(
    r: &Resolver,
    item: Option<&Item>,
    name: &str,
    location: i32,
) -> Option<Reference> {
    if !r.collects_references() {
        return None;
    }
    let (source, column) = item?.column_source(name)?;
    Some(Reference::Column {
        location,
        span: r.name_span(location),
        source,
        column,
    })
}

fn record(r: &mut Resolver, reference: Option<Reference>) {
    if let Some(reference) = reference {
        r.record(reference);
    }
}

/// Postgres reads `t.name` as the function call `name(t)` when `t` has no column `name`
/// ([`transformColumnRef`] falls back to [`ParseFuncOrColumn`]).
///
/// [`transformColumnRef`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L509
/// [`ParseFuncOrColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L90
fn may_be_function_call(r: &Resolver, name: &str) -> bool {
    match r.catalog.functions(None, name, r.search_path) {
        Lookup::Found(overloads) => overloads
            .iter()
            .any(|overload| overload.min_args <= 1 && overload.max_args.is_none_or(|max| max >= 1)),
        Lookup::Missing => false,
        Lookup::Unknown => true,
    }
}
