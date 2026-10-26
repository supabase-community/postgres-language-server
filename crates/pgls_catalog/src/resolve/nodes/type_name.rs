use pgls_query::protobuf::TypeName;
use pgls_query_ext::utils::{QualifiedName, qualified_name};

use super::Resolver;
use crate::lookup::Lookup;
use crate::resolve::FindingKind;

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
    if n.names.is_empty() {
        return;
    }
    let Some(QualifiedName { schema, name, .. }) = qualified_name(&n.names) else {
        r.depends_on_file();
        return;
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
