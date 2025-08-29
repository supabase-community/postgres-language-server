use pgt_schema_cache::SchemaCache;
use pgt_text_size::TextSize;
use pgt_treesitter::TreeSitterContextParams;

use crate::{
    contextual_priority::prioritize_by_context, hovered_item::HoverItem, hovered_node::HoveredNode,
    to_markdown::format_hover_markdown,
};

mod contextual_priority;
mod hovered_item;
mod hovered_node;
mod to_markdown;

pub struct OnHoverParams<'a> {
    pub position: TextSize,
    pub schema_cache: &'a SchemaCache,
    pub stmt_sql: &'a str,
    pub ast: Option<&'a pgt_query::NodeEnum>,
    pub ts_tree: &'a tree_sitter::Tree,
}

pub fn on_hover(params: OnHoverParams) -> Vec<String> {
    let ctx = pgt_treesitter::context::TreesitterContext::new(TreeSitterContextParams {
        position: params.position,
        text: params.stmt_sql,
        tree: params.ts_tree,
    });

    if let Some(hovered_node) = HoveredNode::get(&ctx) {
        let items: Vec<HoverItem> = match hovered_node {
            HoveredNode::Table(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(n) => params
                    .schema_cache
                    .find_tables(n.as_str(), None)
                    .into_iter()
                    .map(|t| HoverItem::from(t))
                    .collect(),

                hovered_node::NodeIdentification::SchemaAndName((s, n)) => params
                    .schema_cache
                    .find_tables(n.as_str(), Some(&s))
                    .into_iter()
                    .map(HoverItem::from)
                    .collect(),

                hovered_node::NodeIdentification::SchemaAndTableAndName(_) => vec![],
            },

            HoveredNode::Column(node_identification) => match node_identification {
                hovered_node::NodeIdentification::Name(column_name) => params
                    .schema_cache
                    .find_cols(&column_name, None, None)
                    .into_iter()
                    .map(HoverItem::from)
                    .collect(),

                hovered_node::NodeIdentification::SchemaAndName((table_or_alias, column_name)) => {
                    params
                        .schema_cache
                        .find_cols(&column_name, Some(&table_or_alias), None)
                        .into_iter()
                        .map(HoverItem::from)
                        .collect()
                }

                hovered_node::NodeIdentification::SchemaAndTableAndName(_) => vec![],
            },

            _ => todo!(),
        };

        prioritize_by_context(items, &ctx)
            .into_iter()
            .map(|item| format_hover_markdown(&item))
            .filter_map(Result::ok)
            .collect()
    } else {
        Default::default()
    }
}
