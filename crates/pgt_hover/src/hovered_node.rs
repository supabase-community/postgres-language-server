#[derive(Debug)]
pub(crate) enum NodeIdentification {
    Name(String),
    SchemaAndName((String, String)),
    #[allow(unused)]
    SchemaAndTableAndName((String, String, String)),
}

#[allow(unused)]
#[derive(Debug)]
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
    pub(crate) fn get(ctx: &pgt_treesitter::context::TreesitterContext) -> Option<Self> {
        let node_content = ctx.get_node_under_cursor_content()?;

        let under_cursor = ctx.node_under_cursor.as_ref()?;

        match under_cursor.kind() {
            "identifier" if ctx.matches_ancestor_history(&["relation", "object_reference"]) => {
                if let Some(schema) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::Table(NodeIdentification::SchemaAndName((
                        schema.clone(),
                        node_content,
                    ))))
                } else {
                    Some(HoveredNode::Table(NodeIdentification::Name(node_content)))
                }
            }
            "identifier" if ctx.matches_ancestor_history(&["field"]) => {
                if let Some(table_or_alias) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::Column(NodeIdentification::SchemaAndName((
                        table_or_alias.clone(),
                        node_content,
                    ))))
                } else {
                    Some(HoveredNode::Column(NodeIdentification::Name(node_content)))
                }
            }
            "identifier" if ctx.matches_ancestor_history(&["invocation", "object_reference"]) => {
                if let Some(schema) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::Function(NodeIdentification::SchemaAndName((
                        schema.clone(),
                        node_content,
                    ))))
                } else {
                    Some(HoveredNode::Function(NodeIdentification::Name(
                        node_content,
                    )))
                }
            }
            "identifier" if ctx.matches_ancestor_history(&["alter_role"]) => {
                Some(HoveredNode::Role(NodeIdentification::Name(node_content)))
            }
            "revoke_role" | "grant_role" | "policy_role" => {
                Some(HoveredNode::Role(NodeIdentification::Name(node_content)))
            }

            // quoted columns
            "literal" if ctx.matches_ancestor_history(&["select_expression", "term"]) => {
                Some(HoveredNode::Column(NodeIdentification::Name(node_content)))
            }

            "policy_table" | "revoke_table" | "grant_table" => {
                if let Some(schema) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::Table(NodeIdentification::SchemaAndName((
                        schema.clone(),
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
