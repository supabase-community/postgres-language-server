//! Facts about a single type: its base type, category, and array element, and the
//! conversion between type names and type identities.

use crate::Catalog;
use crate::typing::{Decision, Type, TypeId, TypedColumn};
use crate::{Lookup, TypeInfo, TypeKind};
use pgls_query::protobuf;
use pgls_query_ext::utils::QualifiedName;

/// The type with this identity. Unknown if it is missing too, as an identity always comes from
/// the catalog.
pub(super) fn info(c: &Catalog, id: &TypeId) -> Decision<TypeInfo> {
    match c.type_by_id(id) {
        Lookup::Found(t) => Decision::Known(t),
        _ => Decision::Unknown,
    }
}

/// The identity of the builtin type `pg_catalog.<name>`, `None` if it certainly does not exist.
/// Unlike [`Catalog::builtin_type`], this tells a missing type from an unknown one.
pub(super) fn lookup_builtin(c: &Catalog, name: &str) -> Decision<Option<TypeId>> {
    c.type_(Some("pg_catalog"), name, &[])
        .into_decision()
        .map(|t| t.and_then(|t| t.id))
}

/// The most domains [`base_type`] follows. Postgres' catalog has no cycles, but a `SchemaCache`
/// from JSON may.
const BASE_TYPE_DOMAINS_MAX: usize = 64;

/// Port of [`getBaseType`]: the type a domain is based on, through nested domains.
///
/// [`getBaseType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/cache/lsyscache.c#L2823
pub fn base_type(c: &Catalog, id: &TypeId) -> Decision<TypeId> {
    let mut current = id.clone();
    for _ in 0..BASE_TYPE_DOMAINS_MAX {
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
    tracing::warn!(
        "The base type of {id:?} is more than {BASE_TYPE_DOMAINS_MAX} domains away; the catalog \
         may have a cycle of domains"
    );
    Decision::Unknown
}

/// The `typcategory` and `typispreferred` of a type. Port of [`get_type_category_preferred`],
/// which [`TypeCategory`] and [`IsPreferredType`] read.
///
/// [`get_type_category_preferred`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/cache/lsyscache.c#L3012
/// [`TypeCategory`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2979
/// [`IsPreferredType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2998
pub(crate) fn type_category(c: &Catalog, id: &TypeId) -> Decision<(char, bool)> {
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
pub(crate) fn is_array(c: &Catalog, id: &TypeId) -> Decision<bool> {
    match info(c, id) {
        Decision::Known(t) => Decision::Known(t.element.is_some()),
        Decision::Unknown => Decision::Unknown,
    }
}

/// Port of [`get_element_type`]: the element type of a true array type.
///
/// [`get_element_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/cache/lsyscache.c#L3061
pub(crate) fn array_element(c: &Catalog, id: &TypeId) -> Decision<Option<TypeId>> {
    match info(c, id) {
        Decision::Known(t) => Decision::Known(t.element),
        Decision::Unknown => Decision::Unknown,
    }
}

/// A serial pseudo-type. In a column of `CREATE TABLE` or `ALTER TABLE ... ADD COLUMN`,
/// Postgres turns it into an integer column with a sequence as its default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialType {
    /// `smallserial` or `serial2`.
    SmallSerial,
    /// `serial` or `serial4`.
    Serial,
    /// `bigserial` or `serial8`.
    BigSerial,
}

impl SerialType {
    /// The integer type that the column gets, e.g. `int4` for `serial`.
    pub fn integer_type(self) -> &'static str {
        match self {
            Self::SmallSerial => "int2",
            Self::Serial => "int4",
            Self::BigSerial => "int8",
        }
    }
}

/// The serial pseudo-type that a column type names, if any. Port of the check in
/// [`transformColumnDefinition`]: only a single unqualified name without `%TYPE` is a serial
/// type, so `pg_catalog.serial` is not (and doesn't exist), and a type named `serial` that the
/// user created can't be used for a column. Postgres rejects arrays of a serial type, so
/// `serial[]` is none.
///
/// [`transformColumnDefinition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_utilcmd.c#L604
pub fn serial_type(type_name: &protobuf::TypeName) -> Option<SerialType> {
    let [name] = type_name.names.as_slice() else {
        return None;
    };
    if type_name.pct_type || !type_name.array_bounds.is_empty() {
        return None;
    }
    match pgls_query_ext::utils::string_value(name)? {
        "smallserial" | "serial2" => Some(SerialType::SmallSerial),
        "serial" | "serial4" => Some(SerialType::Serial),
        "bigserial" | "serial8" => Some(SerialType::BigSerial),
        _ => None,
    }
}

/// Resolves a parser type name and returns its identity plus its SETOF flag. Port of
/// [`LookupTypeNameExtended`], without typmods, which are not checked.
///
/// [`LookupTypeNameExtended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_type.c#L73
pub fn normalize_type_name(
    catalog: &Catalog,
    name: &protobuf::TypeName,
    search_path: &[String],
) -> (Option<TypeId>, bool) {
    if name.pct_type {
        return (None, name.setof);
    }
    let Some(QualifiedName {
        schema,
        name: type_name,
        ..
    }) = QualifiedName::of(&name.names)
    else {
        return (None, name.setof);
    };
    let Lookup::Found(info) = catalog.type_(schema, type_name, search_path) else {
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

/// Displays a type like [`format_type_be`] does.
///
/// [`format_type_be`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/adt/format_type.c#L343
pub fn format_type(c: &Catalog, ty: &Type) -> Option<String> {
    format_type_with_search_path(c, ty, &[])
}

/// Formats a type using the supplied path to determine whether qualification is needed.
pub fn format_type_with_search_path(
    c: &Catalog,
    ty: &Type,
    search_path: &[String],
) -> Option<String> {
    match ty {
        Type::UnknownLiteral => Some("unknown".into()),
        Type::Record(_) => Some("record".into()),
        Type::Named(id) => format_named(c, id, search_path),
    }
}

/// The most array element types [`format_named`] follows. Postgres has no arrays of arrays,
/// but a `SchemaCache` from JSON may have a cycle of element types.
const FORMAT_ELEMENTS_MAX: usize = 16;

/// Port of [`format_type_extended`] without typmods: SQL-standard names for built-in types,
/// and the schema only when the type is not visible on the search path.
///
/// [`format_type_extended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/adt/format_type.c#L112
fn format_named(c: &Catalog, id: &TypeId, search_path: &[String]) -> Option<String> {
    let mut id = id.clone();
    let mut dimensions = 0;
    let info = loop {
        let Lookup::Found(info) = c.type_by_id(&id) else {
            return None;
        };
        let Some(element) = info.element else {
            break info;
        };
        if dimensions == FORMAT_ELEMENTS_MAX {
            tracing::warn!(
                "Type {id:?} is more than {FORMAT_ELEMENTS_MAX} element types deep; the catalog \
                 may have a cycle of array types"
            );
            return None;
        }
        dimensions += 1;
        id = element;
    };
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
        || matches!(c.type_(None, &info.name, search_path), Lookup::Found(visible) if visible.id.as_ref() == Some(&id));
    let mut name = if visible {
        standard.to_owned()
    } else {
        format!("{}.{}", info.schema, standard)
    };
    name.push_str(&"[]".repeat(dimensions));
    Some(name)
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::format_type;
    use crate::typing::Type;
    use crate::{Catalog, Snapshot};
    use sqlx::PgPool;

    #[pgls_test_utils::db_test]
    async fn formats_postgres_builtin_types(test_db: PgPool) {
        let snapshot = Snapshot::load(&test_db).await.unwrap();
        let catalog = Catalog::from_snapshot(snapshot);
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

/// Port of [`resolveTargetListUnknowns`]: output columns of unknown type become
/// text. Postgres does this for every query except the source of an INSERT and the branches of a
/// set operation, which coerce the unknowns to their target type instead.
///
/// [`resolveTargetListUnknowns`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L288
pub fn resolve_unknown_outputs(
    catalog: &Catalog,
    columns: Option<Vec<TypedColumn>>,
) -> Option<Vec<TypedColumn>> {
    let mut columns = columns?;
    if columns
        .iter()
        .any(|column| column.ty == Some(Type::UnknownLiteral))
    {
        let text = catalog.builtin_type("text").map(Type::Named);
        for column in &mut columns {
            if column.ty == Some(Type::UnknownLiteral) {
                column.ty = text.clone();
            }
        }
    }
    Some(columns)
}

#[cfg(test)]
mod serial_tests {
    use super::{SerialType, serial_type};
    use pgls_query::NodeEnum;

    fn column_serial_type(column_type: &str) -> Option<SerialType> {
        let sql = format!("create table t (a {column_type})");
        let Some(NodeEnum::CreateStmt(stmt)) = pgls_query::parse(&sql).unwrap().into_root() else {
            panic!("not a create table: {sql}");
        };
        let Some(NodeEnum::ColumnDef(column)) = &stmt.table_elts[0].node else {
            panic!("not a column: {sql}");
        };
        serial_type(column.type_name.as_ref().unwrap())
    }

    #[test]
    fn serial_types_are_single_unqualified_names() {
        for (column_type, expected) in [
            ("smallserial", SerialType::SmallSerial),
            ("serial2", SerialType::SmallSerial),
            ("serial", SerialType::Serial),
            ("serial4", SerialType::Serial),
            ("SERIAL", SerialType::Serial),
            ("bigserial", SerialType::BigSerial),
            ("serial8", SerialType::BigSerial),
        ] {
            assert_eq!(
                column_serial_type(column_type),
                Some(expected),
                "{column_type}"
            );
        }
        for column_type in [
            "int4",
            "pg_catalog.serial",
            "myschema.bigserial",
            "serial[]",
            "\"Serial\"",
        ] {
            assert_eq!(column_serial_type(column_type), None, "{column_type}");
        }
        assert_eq!(SerialType::SmallSerial.integer_type(), "int2");
        assert_eq!(SerialType::Serial.integer_type(), "int4");
        assert_eq!(SerialType::BigSerial.integer_type(), "int8");
    }
}
