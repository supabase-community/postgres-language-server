#[derive(Debug)]
pub struct TypedIdentifier {
    pub schema: Option<String>,
    pub relation: String,
    pub name: String,
    pub type_: String,
}

/// Applies the identifiers to the SQL string by replacing them with their default values.
pub fn apply_identifiers<'a>(
    identifiers: Vec<TypedIdentifier>,
    schema_cache: &'a pgt_schema_cache::SchemaCache,
    cst: &'a tree_sitter::Node<'a>,
    sql: &'a str,
) -> &'a str {
    // TODO
    sql
}
