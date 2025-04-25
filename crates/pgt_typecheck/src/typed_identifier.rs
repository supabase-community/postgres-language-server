#[derive(Debug)]
pub struct Type {
    pub schema: Option<String>,
    pub name: String,
    pub oid: i32,
}

#[derive(Debug)]
pub struct TypedIdentifier {
    pub schema: Option<String>,
    pub relation: Option<String>,
    pub name: String,
    pub type_: Type,
}

impl TypedIdentifier {
    pub fn new(
        schema: Option<String>,
        relation: Option<String>,
        name: String,
        type_: Type,
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
