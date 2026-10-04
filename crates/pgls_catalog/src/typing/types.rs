//! Facts about a single type: its base type, category, and array element, and the
//! conversion between type names and type identities.

use crate::typing::{Decision, Type, TypeId};
use crate::{CatalogView, Lookup, TypeInfo, TypeKind};
use pgls_query::{NodeEnum, protobuf};

pub(super) fn info(c: &dyn CatalogView, id: &TypeId) -> Decision<TypeInfo> {
    match c.type_by_id(id) {
        Lookup::Found(t) => Decision::Known(t),
        _ => Decision::Unknown,
    }
}

/// Port of [`getBaseType`]: the type a domain is based on, through nested domains.
///
/// [`getBaseType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/cache/lsyscache.c#L2823
pub fn base_type(c: &dyn CatalogView, id: &TypeId) -> Decision<TypeId> {
    let mut current = id.clone();
    loop {
        let meta = match info(c, &current) {
            Decision::Known(t) => t,
            Decision::Unknown => return Decision::Unknown,
        };
        match meta.kind {
            Some(TypeKind::Domain) => {}
            Some(_) => return Decision::Known(current),
            None => return Decision::Unknown,
        }
        let Some(base) = meta.base else {
            return Decision::Unknown;
        };
        if base == current {
            return Decision::Known(current);
        }
        current = base;
    }
}

/// The `typcategory` and `typispreferred` of a type. Port of [`get_type_category_preferred`],
/// which [`TypeCategory`] and [`IsPreferredType`] read.
///
/// [`get_type_category_preferred`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/cache/lsyscache.c#L3012
/// [`TypeCategory`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2979
/// [`IsPreferredType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2998
pub fn type_category(c: &dyn CatalogView, id: &TypeId) -> Decision<(char, bool)> {
    match info(c, id) {
        Decision::Known(t) => match (t.category, t.preferred) {
            (Some(category), Some(preferred)) => Decision::Known((category, preferred)),
            _ => Decision::Unknown,
        },
        Decision::Unknown => Decision::Unknown,
    }
}

/// Port of [`type_is_array`]: whether the type has an element type.
///
/// [`type_is_array`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/utils/lsyscache.h#L218
pub fn is_array(c: &dyn CatalogView, id: &TypeId) -> Decision<bool> {
    match info(c, id) {
        Decision::Known(t) => Decision::Known(t.element.is_some()),
        Decision::Unknown => Decision::Unknown,
    }
}

/// Port of [`get_element_type`]: the element type of a true array type.
///
/// [`get_element_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/cache/lsyscache.c#L3061
pub fn array_element(c: &dyn CatalogView, id: &TypeId) -> Decision<Option<TypeId>> {
    match info(c, id) {
        Decision::Known(t) => Decision::Known(t.element),
        Decision::Unknown => Decision::Unknown,
    }
}

/// Resolves a parser type name and returns its identity plus its SETOF flag. Port of
/// [`LookupTypeNameExtended`], without typmods, which are not checked.
///
/// [`LookupTypeNameExtended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_type.c#L73
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

/// Displays a type like [`format_type_be`] does.
///
/// [`format_type_be`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/adt/format_type.c#L343
pub fn format_type(c: &dyn CatalogView, ty: &Type) -> Option<String> {
    format_type_with_search_path(c, ty, &[])
}

/// Formats a type using the supplied path to determine whether qualification is needed.
pub fn format_type_with_search_path(
    c: &dyn CatalogView,
    ty: &Type,
    search_path: &[String],
) -> Option<String> {
    match ty {
        Type::UnknownLiteral => Some("unknown".into()),
        Type::Record(_) => Some("record".into()),
        Type::Named(id) => format_named(c, id, search_path, 0),
    }
}

/// Port of [`format_type_extended`] without typmods: SQL-standard names for built-in types,
/// and the schema only when the type is not visible on the search path.
///
/// [`format_type_extended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/adt/format_type.c#L112
fn format_named(
    c: &dyn CatalogView,
    id: &TypeId,
    search_path: &[String],
    depth: usize,
) -> Option<String> {
    if depth > 16 {
        return None;
    }
    let info = match c.type_by_id(id) {
        Lookup::Found(info) => info,
        _ => return None,
    };
    if let Some(element) = info.element.as_ref() {
        return Some(format!(
            "{}[]",
            format_named(c, element, search_path, depth + 1)?
        ));
    }
    let standard = match info.name.as_str() {
        "int2" => "smallint",
        "int4" => "integer",
        "int8" => "bigint",
        "varchar" => "character varying",
        "bpchar" => "character",
        "bool" => "boolean",
        "float4" => "real",
        "float8" => "double precision",
        "timetz" => "time with time zone",
        "time" => "time without time zone",
        "timestamptz" => "timestamp with time zone",
        "timestamp" => "timestamp without time zone",
        "bit" => "bit",
        "varbit" => "bit varying",
        "decimal" => "numeric",
        name => name,
    };
    let visible = info.schema == "pg_catalog"
        || matches!(c.type_(None, &info.name, search_path), Lookup::Found(visible) if visible.id.as_ref() == Some(id));
    Some(if visible {
        standard.to_owned()
    } else {
        format!("{}.{}", info.schema, standard)
    })
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::format_type;
    use crate::typing::Type;
    use crate::{Catalog, CatalogBase, Snapshot};
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn formats_postgres_builtin_types(test_db: PgPool) {
        let snapshot = Snapshot::load(&test_db).await.unwrap();
        let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot)))));
        for (oid, expected) in [
            (23, "integer"),
            (20, "bigint"),
            (25, "text"),
            (1184, "timestamp with time zone"),
            (1007, "integer[]"),
        ] {
            assert_eq!(
                format_type(&catalog, &Type::Named(crate::typing::TypeId::Snapshot(oid)))
                    .as_deref(),
                Some(expected)
            );
        }
        assert_eq!(
            format_type(&catalog, &Type::UnknownLiteral).as_deref(),
            Some("unknown")
        );
        assert_eq!(
            format_type(&catalog, &Type::Record(vec![])).as_deref(),
            Some("record")
        );
    }
}
