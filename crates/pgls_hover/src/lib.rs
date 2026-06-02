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
pub fn on_hover(params: OnHoverParams) -> Option<Vec<String>> {
    let ctx = pgls_treesitter::context::TreesitterContext::new(TreeSitterContextParams {
        position: params.position,
        text: params.stmt_sql,
        tree: params.ts_tree,
    });

    if let Some(hovered_node) = HoveredNode::get(&ctx) {
        let items: Vec<Hoverable> = match hovered_node {
            HoveredNode::Table(node_identification) => match node_identification {
                (None, n) => params
                    .schema_cache
                    .find_tables(n.as_str(), None)
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                (Some(s), n) => params
                    .schema_cache
                    .find_tables(n.as_str(), Some(&s))
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),
            },

            HoveredNode::Column(node_identification) => match node_identification {
                (None, None, column_name) => params
                    .schema_cache
                    .find_cols(&column_name, None, None)
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                (None, Some(table_or_alias), column_name) => {
                    // resolve alias to actual table name if needed
                    let actual_table = ctx
                        .get_mentioned_table_for_alias(table_or_alias.as_str())
                        .map(|s| s.as_str())
                        .unwrap_or(&table_or_alias);

                    params
                        .schema_cache
                        .find_cols(&column_name, Some(actual_table), None)
                        .into_iter()
                        .map(Hoverable::from)
                        .collect()
                }

                (Some(schema), Some(table), column_name) => params
                    // no need to resolve table; there can't be both schema qualification and an alias.
                    .schema_cache
                    .find_cols(&column_name, Some(&table), Some(&schema))
                    .into_iter()
                    .map(Hoverable::from)
                    .collect(),

                _ => vec![],
            },

            HoveredNode::Function(node_identification) => {
                let (maybe_schema, function_name) = node_identification;
                params
                    .schema_cache
                    .find_functions(&function_name, maybe_schema.as_deref())
                    .into_iter()
                    .map(Hoverable::from)
                    .collect()
            }

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

            HoveredNode::PostgresType(node_identification) => match node_identification {
                (None, type_name) => params
                    .schema_cache
                    .find_type(&type_name, None)
                    .map(Hoverable::from)
                    .map(|s| vec![s])
                    .unwrap_or_default(),

                (Some(schema), type_name) => params
                    .schema_cache
                    .find_type(&type_name, Some(schema.as_str()))
                    .map(Hoverable::from)
                    .map(|s| vec![s])
                    .unwrap_or_default(),
            },

            HoveredNode::Policy(_) | HoveredNode::Trigger(_) => return None,
        };

        let markdown_blocks: Vec<String> = prioritize_by_context(items, &ctx)
            .into_iter()
            .map(|item| format_hover_markdown(&item, params.schema_cache))
            .filter_map(Result::ok)
            .collect();

        (!markdown_blocks.is_empty()).then_some(markdown_blocks)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_over_drop_policy_name_returns_none() {
        let query =
            r#"drop policy if exists "Av{}atar images are publicly readable" on storage.objects;"#;
        let position = query.find("{}").unwrap();
        let sql = query.replace("{}", "");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(&sql, None).unwrap();
        let schema_cache = SchemaCache::default();

        let hover = on_hover(OnHoverParams {
            position: TextSize::new(position as u32),
            schema_cache: &schema_cache,
            stmt_sql: &sql,
            ast: None,
            ts_tree: &tree,
        });

        assert!(hover.is_none());
    }
}
