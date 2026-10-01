use crate::typing::{Type, TypeId};
use crate::{CatalogView, Lookup};

/// PostgreSQL `format_type_be`-style display for inferred types.
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

// Port of PostgreSQL `format_type_be` for catalog types used by diagnostics.
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
