use pgt_treesitter::WrappingClause;

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
    PostgresType(NodeIdentification),
}

impl HoveredNode {
    pub(crate) fn get(ctx: &pgt_treesitter::context::TreesitterContext) -> Option<Self> {
        let node_content = ctx.get_node_under_cursor_content()?;

        let under_cursor = ctx.node_under_cursor.as_ref()?;

        match under_cursor.kind() {
            "identifier" if ctx.matches_ancestor_history(&["relation", "object_reference"]) => {
                let num_sibs = ctx.num_siblings();
                if ctx.node_under_cursor_is_nth_child(1) && num_sibs > 0 {
                    return Some(HoveredNode::Schema(NodeIdentification::Name(node_content)));
                }

                if let Some(schema) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::Table(NodeIdentification::SchemaAndName((
                        schema.clone(),
                        node_content,
                    ))))
                } else {
                    Some(HoveredNode::Table(NodeIdentification::Name(node_content)))
                }
            }

            "identifier"
                if ctx.matches_ancestor_history(&["object_reference"])
                    && ctx.wrapping_clause_type.as_ref().is_some_and(|clause| {
                        matches!(
                            clause,
                            WrappingClause::AlterPolicy
                                | WrappingClause::CreatePolicy
                                | WrappingClause::DropPolicy
                        )
                    }) =>
            {
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

            "identifier" if ctx.matches_one_of_ancestors(&["alter_role", "policy_to_role"]) => {
                Some(HoveredNode::Role(NodeIdentification::Name(node_content)))
            }

            "identifier"
                if (
                    // hover over custom type in create table, returns…
                    (ctx.matches_ancestor_history(&["type", "object_reference"])
                    && ctx.node_under_cursor_is_within_field_name("custom_type"))

                    // hover over type in select clause etc…                    
                    || (ctx
                        .matches_ancestor_history(&["field_qualifier", "object_reference"])
                        && ctx.before_cursor_matches_kind(&["("])))

                    // make sure we're not checking against an alias
                    && ctx
                        .get_mentioned_table_for_alias(
                            node_content.replace('(', "").replace(')', "").as_str(),
                        )
                        .is_none() =>
            {
                let sanitized = node_content.replace('(', "").replace(')', "");
                if let Some(schema) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::PostgresType(
                        NodeIdentification::SchemaAndName((schema.clone(), sanitized)),
                    ))
                } else {
                    Some(HoveredNode::PostgresType(NodeIdentification::Name(
                        sanitized,
                    )))
                }
            }

            "revoke_role" | "grant_role" | "policy_role" => {
                Some(HoveredNode::Role(NodeIdentification::Name(node_content)))
            }

            // quoted columns
            "literal" if ctx.matches_ancestor_history(&["select_expression", "term"]) => {
                Some(HoveredNode::Column(NodeIdentification::Name(node_content)))
            }

            "revoke_table" | "grant_table" => {
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
