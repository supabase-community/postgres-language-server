use pgt_schema_cache::SchemaCache;
use pgt_text_size::TextSize;

mod hovered_node;
mod to_markdown;

pub struct OnHoverParams<'a> {
    pub position: TextSize,
    pub schema_cache: &'a SchemaCache,
    pub ast: Option<&'a pgt_query_ext::NodeEnum>,
    pub ts_tree: &'a tree_sitter::Tree,
}

pub fn on_hover(_params: OnHoverParams) -> Vec<String> {
    // needs to find the right element(s) in the schema_cache
    // then, we should map the schema_cache items into markdown strings.

    vec![]
}
