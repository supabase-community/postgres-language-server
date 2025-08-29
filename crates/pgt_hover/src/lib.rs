use pgt_schema_cache::SchemaCache;
use pgt_text_size::TextSize;
use pgt_treesitter::TreeSitterContextParams;

use crate::{hovered_node::HoveredNode, to_markdown::format_hover_markdown};

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
        match hovered_node {
            HoveredNode::Table(node_identification) => {
                let possible_tables = match node_identification {
                    hovered_node::NodeIdentification::Name(n) => {
                        params.schema_cache.find_tables(n.as_str(), None)
                    }
                    hovered_node::NodeIdentification::SchemaAndName((s, n)) => params
                        .schema_cache
                        .find_tables(n.as_str(), Some(s.as_str())),
                    hovered_node::NodeIdentification::SchemaAndTableAndName(_) => vec![],
                };

                possible_tables
                    .into_iter()
                    .filter_map(|t| match format_hover_markdown(t) {
                        Ok(markdown) => Some(markdown),
                        Err(_) => None,
                    })
                    .collect()
            }

            HoveredNode::Column(node_identification) => {
                let possible_columns = match node_identification {
                    hovered_node::NodeIdentification::Name(column_name) => {
                        params.schema_cache.find_cols(&column_name, None, None)
                    }

                    hovered_node::NodeIdentification::SchemaAndName((
                        table_or_alias,
                        column_name,
                    )) => params
                        .schema_cache
                        .find_cols(&column_name, Some(&table_or_alias), None),

                    hovered_node::NodeIdentification::SchemaAndTableAndName(_) => vec![],
                };

                possible_columns
                    .into_iter()
                    .filter_map(|c| match format_hover_markdown(c) {
                        Ok(markdown) => Some(markdown),
                        Err(_) => None,
                    })
                    .collect()
            }

            _ => todo!(),
        }
    } else {
        Default::default()
    }
}
