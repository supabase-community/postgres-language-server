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
            HoveredNode::Table(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(n) => params
                    .schema_cache
                    .find_tables(n.as_str(), None)
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                hovered_node::NodeIdentification::SchemaAndName((s, n)) => params
                    .schema_cache
                    .find_tables(n.as_str(), Some(&s))
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                _ => vec![],
            },

            HoveredNode::Column(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(column_name) => params
                    .schema_cache
                    .find_cols(&column_name, None, None)
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                hovered_node::NodeIdentification::SchemaAndName((table_or_alias, column_name)) => {
                    // resolve alias to actual table name if needed
                    let actual_table = ctx
                        .get_mentioned_table_for_alias(table_or_alias.as_str())
                        .map(|s| s.as_str())
                        .unwrap_or(table_or_alias.as_str());

                    params
                        .schema_cache
                        .find_cols(&column_name, Some(actual_table), None)
                        .into_iter()
                        .map(Hoverable::from)
                        .collect()
                }

                _ => vec![],
            },

            HoveredNode::Function(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(function_name) => params
                    .schema_cache
                    .find_functions(&function_name, None)
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                hovered_node::NodeIdentification::SchemaAndName((schema, function_name)) => params
                    .schema_cache
                    .find_functions(&function_name, Some(&schema))
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                _ => vec![],
            },

            HoveredNode::Role(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(role_name) => params
                    .schema_cache
                    .find_roles(&role_name)
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                _ => vec![],
            },

            HoveredNode::Schema(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(schema_name) => params
                    .schema_cache
                    .find_schema(&schema_name)
                    .map(Hoverable::from)
                    .map(|s| vec![s])
                    .unwrap_or_default(),

                _ => vec![],
            },

            HoveredNode::PostgresType(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(type_name) => params
                    .schema_cache
                    .find_type(&type_name, None)
                    .map(Hoverable::from)
                    .map(|s| vec![s])
                    .unwrap_or_default(),

                hovered_node::NodeIdentification::SchemaAndName((schema, type_name)) => params
                    .schema_cache
                    .find_type(&type_name, Some(schema.as_str()))
                    .map(Hoverable::from)
                    .map(|s| vec![s])
                    .unwrap_or_default(),

                _ => vec![],
            },

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
