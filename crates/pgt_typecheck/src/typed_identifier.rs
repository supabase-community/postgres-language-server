#[derive(Debug)]
pub struct TypedIdentifier {
    pub schema: Option<String>,
    pub relation: Option<String>,
    pub name: String,
    pub type_: (Option<String>, String),
}

impl TypedIdentifier {
    pub fn new(
        schema: Option<String>,
        relation: Option<String>,
        name: String,
        type_: (Option<String>, String),
    ) -> Self {
        TypedIdentifier {
            schema,
            relation,
            name,
            type_,
        }
    }

    pub fn default_value(&self, schema_cache: &pgt_schema_cache::SchemaCache) -> String {
        "NULL".to_string()
    }
}

/// Applies the identifiers to the SQL string by replacing them with their default values.
pub fn apply_identifiers<'a>(
    identifiers: Vec<TypedIdentifier>,
    schema_cache: &'a pgt_schema_cache::SchemaCache,
    cst: &'a tree_sitter::Tree,
    sql: &'a str,
) -> &'a str {
    // TODO
    println!("Applying identifiers to SQL: {}", sql);
    println!("Identifiers: {:?}", identifiers);
    println!("CST: {:#?}", cst);

    sql
}

#[cfg(test)]
mod tests {
    use pgt_test_utils::test_database::get_new_test_db;
    use sqlx::Executor;

    #[tokio::test]
    async fn test_apply_identifiers() {
        let input = "select v_test + fn_name.custom_type.v_test2 + $3 + test.field;";

        let test_db = get_new_test_db().await;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let schema_cache = pgt_schema_cache::SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let root = pgt_query_ext::parse(input).unwrap();
        let tree = parser.parse(input, None).unwrap();

        println!("Parsed SQL: {:?}", root);
        println!("Parsed CST: {:?}", tree);

        // let mut parameters = Vec::new();

        enum Parameter {
            Identifier {
                range: (usize, usize),
                name: (Option<String>, String),
            },
            Parameter {
                range: (usize, usize),
                idx: usize,
            },
        }

        let mut c = tree.walk();

        'outer: loop {
            // 0. Add the current node to the map.
            println!("Current node: {:?}", c.node());
            match c.node().kind() {
                "identifier" => {
                    println!(
                        "Found identifier: {:?}",
                        c.node().utf8_text(input.as_bytes()).unwrap()
                    );
                }
                "parameter" => {
                    println!(
                        "Found parameter: {:?}",
                        c.node().utf8_text(input.as_bytes()).unwrap()
                    );
                }
                "object_reference" => {
                    println!(
                        "Found object reference: {:?}",
                        c.node().utf8_text(input.as_bytes()).unwrap()
                    );

                    // let source = self.text;
                    // ts_node.utf8_text(source.as_bytes()).ok().map(|txt| {
                    //     if SanitizedCompletionParams::is_sanitized_token(txt) {
                    //         NodeText::Replaced
                    //     } else {
                    //         NodeText::Original(txt)
                    //     }
                    // })
                }
                _ => {}
            }

            // 1. Go to its child and continue.
            if c.goto_first_child() {
                continue 'outer;
            }

            // 2. We've reached a leaf (node without a child). We will go to a sibling.
            if c.goto_next_sibling() {
                continue 'outer;
            }

            // 3. If there are no more siblings, we need to go back up.
            'inner: loop {
                // 4. Check if we've reached the root node. If so, we're done.
                if !c.goto_parent() {
                    break 'outer;
                }
                // 5. Go to the previous node's sibling.
                if c.goto_next_sibling() {
                    // And break out of the inner loop.
                    break 'inner;
                }
            }
        }
    }
}
