use pgt_schema_cache::SchemaCache;
use pgt_text_size::TextSize;

use crate::{hovered_node::HoveredNode, to_markdown::ToHoverMarkdown};

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
    if let Some(hovered_node) = HoveredNode::get(params.position, params.stmt_sql, params.ts_tree) {
        match hovered_node {
            HoveredNode::Table(node_identification) => {
                let table = match node_identification {
                    hovered_node::NodeIdentification::Name(n) => {
                        params.schema_cache.find_table(n.as_str(), None)
                    }
                    hovered_node::NodeIdentification::SchemaAndName((s, n)) => {
                        params.schema_cache.find_table(n.as_str(), Some(s.as_str()))
                    }
                    hovered_node::NodeIdentification::SchemaAndTableAndName(_) => None,
                };

                table
                    .map(|t| {
                        let mut markdown = String::new();
                        match t.to_hover_markdown(&mut markdown) {
                            Ok(_) => vec![markdown],
                            Err(_) => vec![],
                        }
                    })
                    .unwrap_or(vec![])
            }

            _ => todo!(),
        }
    } else {
        Default::default()
    }
}
