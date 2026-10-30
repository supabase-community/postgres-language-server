use pgls_query::{NodeEnum, NodeRef, protobuf::TypeName};

use super::{Resolver, string::string_values};
use crate::lookup::Lookup;
use crate::resolve::{FindingKind, Reference};

/// Types whose input is the name of a catalog object, which we don't resolve.
const OBJECT_NAME_TYPES: &[&str] = &[
    "regclass",
    "regcollation",
    "regconfig",
    "regdictionary",
    "regnamespace",
    "regoper",
    "regoperator",
    "regproc",
    "regprocedure",
    "regrole",
    "regtype",
];

pub(super) fn resolve_type_name(r: &mut Resolver, n: &TypeName) {
    if n.pct_type {
        r.depends_on_file();
        return;
    }
    let Some(names) = string_values(&n.names) else {
        r.depends_on_file();
        return;
    };
    let (schema, name) = match names.as_slice() {
        [] => return,
        [name] => (None, name.as_str()),
        [schema, name] => (Some(schema.as_str()), name.as_str()),
        _ => {
            r.depends_on_file();
            return;
        }
    };
    if let Some(schema) = schema
        && r.check_schema(schema, n.location)
    {
        return;
    }

    match r.catalog.type_(schema, name, r.search_path) {
        Lookup::Found(type_info) => {
            r.uses(type_info.origin);
            if OBJECT_NAME_TYPES.contains(&name) {
                r.depends_on_file();
            }
            if r.collects_references() {
                r.record(Reference::Type {
                    location: n.location,
                    span: r.type_name_span(n.location),
                    type_info,
                });
            }
        }
        Lookup::Missing => r.report(
            FindingKind::UnknownType {
                schema: schema.map(str::to_owned),
                name: name.to_owned(),
            },
            n.location,
        ),
        Lookup::Unknown => r.depends_on_file(),
    }
}

/// Records the types of the type names in `node` that the resolver doesn't resolve otherwise,
/// like the column types of `CREATE TABLE` or the signature of `CREATE FUNCTION`, when
/// references are being collected. Only types that are found get a reference, and nothing is
/// reported.
pub(super) fn record_type_names(r: &mut Resolver, node: &NodeEnum) {
    if !r.collects_references() {
        return;
    }
    for node in node.iter() {
        let NodeRef::TypeName(n) = node else {
            continue;
        };
        if n.pct_type {
            continue;
        }
        let Some(names) = string_values(&n.names) else {
            continue;
        };
        let (schema, name) = match names.as_slice() {
            [name] => (None, name.as_str()),
            [schema, name] => (Some(schema.as_str()), name.as_str()),
            _ => continue,
        };
        if let Lookup::Found(type_info) = r.catalog.type_(schema, name, r.search_path) {
            r.record(Reference::Type {
                location: n.location,
                span: r.type_name_span(n.location),
                type_info,
            });
        }
    }
}
