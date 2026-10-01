use crate::typing::TypeId;
use crate::{CatalogView, Lookup, TypeInfo};
use pgls_query::{NodeEnum, protobuf};

/// Resolves a parser type name and returns its identity plus its SETOF flag.
pub fn normalize_type_name(
    catalog: &dyn CatalogView,
    name: &protobuf::TypeName,
    search_path: &[String],
) -> (Option<TypeId>, bool) {
    if name.pct_type {
        return (None, name.setof);
    }
    let mut names: Vec<String> = name
        .names
        .iter()
        .filter_map(|n| match n.node.as_ref()? {
            NodeEnum::String(s) => Some(s.sval.clone()),
            _ => None,
        })
        .collect();
    let Some(type_name) = names.pop() else {
        return (None, name.setof);
    };
    let schema = names.pop();
    let Lookup::Found(info) = catalog.type_(schema.as_deref(), &type_name, search_path) else {
        return (None, name.setof);
    };
    let mut id = info.id;
    if !name.array_bounds.is_empty() {
        let Some(current) = id.as_ref() else {
            return (None, name.setof);
        };
        let Lookup::Found(info) = catalog.type_by_id(current) else {
            return (None, name.setof);
        };
        id = info.array;
    }
    (id, name.setof)
}

/// Converts a snapshot type identity to the public typing representation.
pub fn type_from_info(info: &TypeInfo) -> Option<crate::typing::Type> {
    info.id.clone().map(crate::typing::Type::Named)
}
