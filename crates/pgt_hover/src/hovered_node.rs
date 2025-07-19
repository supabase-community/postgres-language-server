use pgt_text_size::TextSize;
use pgt_treesitter::TreeSitterContextParams;

pub(crate) enum NodeIdentification {
    Name(String),
    SchemaAndName((String, String)),
    SchemaAndTableAndName((String, String, String)),
}

pub(crate) enum HoveredNode {
    Schema(NodeIdentification),
    Table(NodeIdentification),
    Function(NodeIdentification),
    Column(NodeIdentification),
    Policy(NodeIdentification),
    Trigger(NodeIdentification),
    Role(NodeIdentification),
}

impl HoveredNode {
    pub(crate) fn get(position: TextSize, text: &str, tree: &tree_sitter::Tree) -> Option<Self> {
        let ctx = pgt_treesitter::context::TreesitterContext::new(TreeSitterContextParams {
            position,
            text,
            tree,
        });

        let node_content = ctx.get_node_under_cursor_content()?;
        let under_node = ctx.node_under_cursor?;

        match under_node.kind() {
            "relation" => {
                if let Some(schema) = ctx.schema_or_alias_name {
                    Some(HoveredNode::Table(NodeIdentification::SchemaAndName((
                        schema,
                        node_content,
                    ))))
                } else {
                    Some(HoveredNode::Table(NodeIdentification::Name(node_content)))
                }
            }
            _ => None,
        }
    }
}
