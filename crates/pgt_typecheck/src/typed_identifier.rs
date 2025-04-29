use pgt_treesitter_queries::{TreeSitterQueriesExecutor, queries::ParameterMatch};

/// A typed identifier is a parameter that has a type associated with it.
/// It is used to replace parameters within the SQL string.
#[derive(Debug)]
pub struct TypedIdentifier {
    /// The path of the parameter, usually the name of the function.
    /// This is because `fn_name.arg_name` is a valid reference within a SQL function.
    pub path: String,
    /// The name of the argument
    pub name: String,
    /// The type of the argument with schema and name
    pub type_: Identifier,
}

type Identifier = (Option<String>, String);

/// Applies the identifiers to the SQL string by replacing them with their default values.
pub fn apply_identifiers<'a>(
    identifiers: Vec<TypedIdentifier>,
    schema_cache: &'a pgt_schema_cache::SchemaCache,
    cst: &'a tree_sitter::Tree,
    sql: &'a str,
) -> &'a str {
    let mut executor = TreeSitterQueriesExecutor::new(cst.root_node(), sql);

    executor.add_query_results::<ParameterMatch>();

    // we need the range and type of each field
    let results = executor
        .get_iter(None)
        .filter_map(|q| {
            let m: &ParameterMatch = q.try_into().ok()?;

            let path = m.get_path(sql);
            let parts = path.split(".").collect::<Vec<_>>();

            // find the identifier and its index
            // if it starts with $ it is a parameter, e.g. `$2` targets the second parameter
            let (ident, idx) = if parts.len() == 1 && parts[0].starts_with("$") {
                let idx = parts[0][1..].parse::<usize>().ok()?;

                let ident = identifiers.get(idx - 1)?;

                (ident, idx)
            } else {
                // If it is not a parameter, its the path to the identifier
                // e.g. `fn_name.custom_type.v_test2` or `custom_type.v_test3` or just `v_test4`
                // Note that we cannot know if its `fn_name.arg_name` or `arg_name.field_name` (for
                // composite types).
                identifiers.iter().find_map(|i| {
                    let (idx, _part) = parts.iter().enumerate().find(|(_idx, p)| **p == i.name)?;

                    Some((i, idx))
                })?
            };

            println!("Found identifier: {:?}", ident);

            // now resolve its type
            let type_ = if idx < parts.len() - 1 {
                // special case: composite types
                let (schema, name) = &ident.type_;

                let schema_type = schema_cache
                    .types
                    .iter()
                    .find(|t| schema.as_ref().is_none_or(|s| t.schema == *s) && t.name == *name)?;

                let field_name = parts.last().unwrap();

                let field = schema_type
                    .attributes
                    .attrs
                    .iter()
                    .find(|a| a.name == *field_name)?;

                let field_type = schema_cache.types.iter().find(|t| t.id == field.type_id)?;

                (Some(field_type.schema.as_str()), field_type.name.as_str())
            } else {
                // find schema of the type
                let schema = ident.type_.0.as_deref().or_else(|| {
                    schema_cache
                        .find_type(&ident.type_.1, None)
                        .map(|t| t.schema.as_str())
                });

                (schema, ident.type_.1.as_str())
            };

            Some((m.get_byte_range(), type_))
        })
        .collect::<Vec<_>>();

    println!("Results: {:?}", results);

    // now resolve the default values
    // for enums we need to fetch the values
    // for everything else we implement a default value generator
    // we then replace the identifier with the default value
    // we will have an issue with enum values that are longer than the original identifier, e.g. $1
    // but for the rest we can simply fill up the space with spaces.
    // we might be able to use NULL for some types or as a fallback.
    // for now, we can simply not expose the location if the default is larger than the identifier

    results.iter().for_each(|(r, type_)| {
        let (schema, name) = type_;

        // if the type not in pg_catalog, its probably an enum and we want to fetch one of its
        // values
    });

    sql
}

#[cfg(test)]
mod tests {
    use pgt_test_utils::test_database::get_new_test_db;
    use sqlx::Executor;

    #[tokio::test]
    async fn test_apply_identifiers() {
        let input = "select v_test + fn_name.custom_type.v_test2 + $3 + custom_type.v_test3 + fn_name.v_test2 + enum_type";

        let identifiers = vec![
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: "v_test".to_string(),
                type_: (None, "int4".to_string()),
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: "custom_type".to_string(),
                type_: (Some("public".to_string()), "custom_type".to_string()),
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: "another".to_string(),
                type_: (None, "numeric".to_string()),
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: "custom_type".to_string(),
                type_: (Some("public".to_string()), "custom_type".to_string()),
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: "v_test2".to_string(),
                type_: (None, "int4".to_string()),
            },
            super::TypedIdentifier {
                path: "fn_name".to_string(),
                name: "enum_type".to_string(),
                type_: (Some("public".to_string()), "enum_type".to_string()),
            },
        ];

        let test_db = get_new_test_db().await;

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

        super::apply_identifiers(identifiers, &schema_cache, &tree, input);
    }
}
