use pgt_schema_cache::PostgresType;
use pgt_treesitter::queries::{ParameterMatch, TreeSitterQueriesExecutor};

/// A typed identifier is a parameter that has a type associated with it.
/// It is used to replace parameters within the SQL string.
#[derive(Debug)]
pub struct TypedIdentifier {
    /// The path of the parameter, usually the name of the function.
    /// This is because `fn_name.arg_name` is a valid reference within a SQL function.
    pub path: String,
    /// The name of the argument
    pub name: Option<String>,
    /// The type of the argument with schema and name
    pub type_: IdentifierType,
}

#[derive(Debug, Clone)]
pub struct IdentifierType {
    pub schema: Option<String>,
    pub name: String,
    pub is_array: bool,
}

/// Applies the identifiers to the SQL string by replacing them with their default values.
pub fn apply_identifiers<'a>(
    identifiers: Vec<TypedIdentifier>,
    schema_cache: &'a pgt_schema_cache::SchemaCache,
    cst: &'a tree_sitter::Tree,
    sql: &'a str,
) -> (String, bool) {
    let mut executor = TreeSitterQueriesExecutor::new(cst.root_node(), sql);

    executor.add_query_results::<ParameterMatch>();

    // Collect all replacements first to avoid modifying the string while iterating
    let replacements: Vec<_> = executor
        .get_iter(None)
        .filter_map(|q| {
            let m: &ParameterMatch = q.try_into().ok()?;
            let path = m.get_path(sql);
            let parts: Vec<_> = path.split('.').collect();

            // Find the matching identifier and its position in the path
            let (identifier, position) = find_matching_identifier(&parts, &identifiers)?;

            // Resolve the type based on whether we're accessing a field of a composite type
            let type_ = resolve_type(identifier, position, &parts, schema_cache)?;

            Some((m.get_byte_range(), type_, identifier.type_.is_array))
        })
        .collect();

    let mut result = sql.to_string();

    let mut valid_positions = true;

    // Apply replacements in reverse order to maintain correct byte offsets
    for (range, type_, is_array) in replacements.into_iter().rev() {
        let default_value = get_formatted_default_value(type_, is_array);

        // if the default_value is shorter than "range", fill it up with spaces
        let default_value = if default_value.len() < range.end - range.start {
            format!("{:<width$}", default_value, width = range.end - range.start)
        } else {
            default_value
        };

        // check if we can maintain a valid position
        if default_value.len() > range.end - range.start {
            valid_positions = false;
        }

        result.replace_range(range, &default_value);
    }

    (result, valid_positions)
}

/// Format the default value based on the type and whether it's an array
fn get_formatted_default_value(pg_type: &PostgresType, is_array: bool) -> String {
    // Get the base default value for this type
    let default = resolve_default_value(pg_type);

    let default = if default.len() > "NULL".len() {
        // If the default value is longer than "NULL", use "NULL" instead
        "NULL".to_string()
    } else {
        // Otherwise, use the default value
        default
    };

    // For arrays, wrap the default in array syntax
    if is_array {
        format!("'{{{}}}'", default)
    } else {
        default
    }
}

/// Resolve the default value for a given Postgres type.
///
/// * `pg_type`: The type to return the default value for.
pub fn resolve_default_value(pg_type: &PostgresType) -> String {
    // Handle ENUM types by returning the first variant
    if !pg_type.enums.values.is_empty() {
        return format!("'{}'", pg_type.enums.values[0]);
    }

    match pg_type.name.as_str() {
        // Numeric types
        "smallint" | "int2" | "integer" | "int" | "int4" | "bigint" | "int8" | "decimal"
        | "numeric" | "real" | "float4" | "double precision" | "float8" | "smallserial"
        | "serial2" | "serial" | "serial4" | "bigserial" | "serial8" => "0".to_string(),

        // Boolean type
        "boolean" | "bool" => "false".to_string(),

        // Character types
        "character" | "char" | "character varying" | "varchar" | "text" => "''".to_string(),

        // Date/time types
        "date" => "'1970-01-01'".to_string(),
        "time" | "time without time zone" => "'00:00:00'".to_string(),
        "time with time zone" | "timetz" => "'00:00:00+00'".to_string(),
        "timestamp" | "timestamp without time zone" => "'1970-01-01 00:00:00'".to_string(),
        "timestamp with time zone" | "timestamptz" => "'1970-01-01 00:00:00+00'".to_string(),
        "interval" => "'0'".to_string(),

        // JSON types
        "json" | "jsonb" => "'null'".to_string(),

        // UUID
        "uuid" => "'00000000-0000-0000-0000-000000000000'".to_string(),

        // Byte array
        "bytea" => "'\\x'".to_string(),

        // Network types
        "inet" => "'0.0.0.0'".to_string(),
        "cidr" => "'0.0.0.0/0'".to_string(),
        "macaddr" => "'00:00:00:00:00:00'".to_string(),
        "macaddr8" => "'00:00:00:00:00:00:00:00'".to_string(),

        // Monetary type
        "money" => "'0.00'".to_string(),

        // Geometric types
        "point" => "'(0,0)'".to_string(),
        "line" => "'{0,0,0}'".to_string(),
        "lseg" => "'[(0,0),(0,0)]'".to_string(),
        "box" => "'((0,0),(0,0))'".to_string(),
        "path" => "'((0,0),(0,0))'".to_string(),
        "polygon" => "'((0,0),(0,0),(0,0))'".to_string(),
        "circle" => "'<(0,0),0>'".to_string(),

        // Text search types
        "tsvector" => "''".to_string(),
        "tsquery" => "''".to_string(),

        // XML
        "xml" => "''".to_string(),

        // Log sequence number
        "pg_lsn" => "'0/0'".to_string(),

        // Snapshot types
        "txid_snapshot" | "pg_snapshot" => "NULL".to_string(),

        // Fallback for unrecognized types
        _ => "NULL".to_string(),
    }
}

// Helper function to find the matching identifier and its position in the path
fn find_matching_identifier<'a>(
    parts: &[&str],
    identifiers: &'a [TypedIdentifier],
) -> Option<(&'a TypedIdentifier, usize)> {
    // Case 1: Parameter reference (e.g., $2)
    if parts.len() == 1 && parts[0].starts_with('$') {
        let idx = parts[0][1..].parse::<usize>().ok()?;
        let identifier = identifiers.get(idx - 1)?;
        return Some((identifier, idx));
    }

    // Case 2: Named reference (e.g., fn_name.custom_type.v_test2)
    identifiers.iter().find_map(|identifier| {
        let name = identifier.name.as_ref()?;

        parts
            .iter()
            .enumerate()
            .find(|(_idx, part)| **part == name)
            .map(|(idx, _)| (identifier, idx))
    })
}

// Helper function to resolve the type based on the identifier and path
fn resolve_type<'a>(
    identifier: &TypedIdentifier,
    position: usize,
    parts: &[&str],
    schema_cache: &'a pgt_schema_cache::SchemaCache,
) -> Option<&'a PostgresType> {
    if position < parts.len() - 1 {
        // Find the composite type
        let schema_type = schema_cache.types.iter().find(|t| {
            identifier
                .type_
                .schema
                .as_ref()
                .is_none_or(|s| t.schema == *s)
                && t.name == *identifier.type_.name
        })?;

        // Find the field within the composite type
        let field_name = parts.last().unwrap();
        let field = schema_type
            .attributes
            .attrs
            .iter()
            .find(|a| a.name == *field_name)?;

        // Find the field's type
        schema_cache.types.iter().find(|t| t.id == field.type_id)
    } else {
        // Direct type reference
        schema_cache.find_type(&identifier.type_.name, identifier.type_.schema.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Executor, PgPool};

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn test_apply_identifiers(test_db: PgPool) {
        let input = "select v_test + fn_name.custom_type.v_test2 + $3 + custom_type.v_test3 + fn_name.v_test2 + enum_type";

        let identifiers = vec![
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: Some("v_test".to_string()),
                type_: super::IdentifierType {
                    schema: None,
                    name: "int4".to_string(),
                    is_array: false,
                },
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: Some("custom_type".to_string()),
                type_: super::IdentifierType {
                    schema: Some("public".to_string()),
                    name: "custom_type".to_string(),
                    is_array: false,
                },
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: Some("another".to_string()),
                type_: super::IdentifierType {
                    schema: None,
                    name: "numeric".to_string(),
                    is_array: false,
                },
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: Some("custom_type".to_string()),
                type_: super::IdentifierType {
                    schema: Some("public".to_string()),
                    name: "custom_type".to_string(),
                    is_array: false,
                },
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: Some("v_test2".to_string()),
                type_: super::IdentifierType {
                    schema: None,
                    name: "int4".to_string(),
                    is_array: false,
                },
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: Some("enum_type".to_string()),
                type_: super::IdentifierType {
                    schema: Some("public".to_string()),
                    name: "enum_type".to_string(),
                    is_array: false,
                },
            },
        ];

        let setup = r#"
            CREATE TYPE "public"."custom_type" AS (
                v_test2 integer,
                v_test3 integer
            );

            CREATE TYPE "public"."enum_type" AS ENUM (
                'critical',
                'high',
                'default',
                'low',
                'very_low'
            );
        "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let schema_cache = pgt_schema_cache::SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let tree = parser.parse(input, None).unwrap();

        let (sql_out, valid_pos) =
            super::apply_identifiers(identifiers, &schema_cache, &tree, input);

        assert!(valid_pos);
        assert_eq!(
            sql_out,
            // the numeric parameters are filled with 0;
            // all values of the enums are longer than `NULL`, so we use `NULL` instead
            "select 0      + 0                           + 0  + 0                   + 0               + NULL     "
        );
    }
}
