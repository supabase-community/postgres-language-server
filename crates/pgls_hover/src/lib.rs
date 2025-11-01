use pgls_schema_cache::SchemaCache;
use pgls_text_size::TextSize;
use pgls_treesitter::TreeSitterContextParams;

use crate::{
    contextual_priority::prioritize_by_context, hoverables::Hoverable, hovered_node::HoveredNode,
    to_markdown::format_hover_markdown,
};

mod contextual_priority;
mod hoverables;
mod hovered_node;
mod to_markdown;

pub struct OnHoverParams<'a> {
    pub position: TextSize,
    pub schema_cache: &'a SchemaCache,
    pub stmt_sql: &'a str,
    pub ast: Option<&'a pgls_query::NodeEnum>,
    pub ts_tree: &'a tree_sitter::Tree,
}

#[tracing::instrument(level = "debug", skip_all, fields(
    text = params.stmt_sql,
    position = params.position.to_string()
))]
pub fn on_hover(params: OnHoverParams) -> Vec<String> {
    let ctx = pgls_treesitter::context::TreesitterContext::new(TreeSitterContextParams {
        position: params.position,
        text: params.stmt_sql,
        tree: params.ts_tree,
    });

    if let Some(hovered_node) = HoveredNode::get(&ctx) {
        let items: Vec<Hoverable> = match hovered_node {
            HoveredNode::Table((schema, name)) => params
                .schema_cache
                .find_tables(name.as_str(), schema.as_deref())
                .into_iter()
                .map(Hoverable::from)
                .collect(),

            HoveredNode::Column((schema, table_or_alias, column_name)) => {
                let table = table_or_alias.as_ref().and_then(|t| {
                    ctx.get_mentioned_table_for_alias(t.as_str())
                        .map(|s| s.as_str())
                        .or(Some(t.as_str()))
                });

                params
                    .schema_cache
                    .find_cols(&column_name, table, schema.as_deref())
                    .into_iter()
                    .map(Hoverable::from)
                    .collect()
            }

            HoveredNode::Function((schema, function_name)) => params
                .schema_cache
                .find_functions(&function_name, schema.as_deref())
                .into_iter()
                .map(Hoverable::from)
                .collect(),

            HoveredNode::Role(role_name) => params
                .schema_cache
                .find_roles(&role_name)
                .into_iter()
                .map(Hoverable::from)
                .collect(),

            HoveredNode::Schema(schema_name) => params
                .schema_cache
                .find_schema(&schema_name)
                .map(Hoverable::from)
                .map(|s| vec![s])
                .unwrap_or_default(),

            HoveredNode::PostgresType((schema, type_name)) => params
                .schema_cache
                .find_type(&type_name, schema.as_deref())
                .map(Hoverable::from)
                .map(|s| vec![s])
                .unwrap_or_default(),

            _ => todo!(),
        };

        prioritize_by_context(items, &ctx)
            .into_iter()
            .map(|item| format_hover_markdown(&item, params.schema_cache))
            .filter_map(Result::ok)
            .collect()
    } else {
        Default::default()
    }
}
