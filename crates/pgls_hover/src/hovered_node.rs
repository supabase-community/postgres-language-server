use pgls_treesitter::WrappingClause;

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
    pub(crate) fn get(ctx: &pgls_treesitter::context::TreesitterContext) -> Option<Self> {
        let node_content = ctx.get_node_under_cursor_content()?;

        if looks_like_sql_param(node_content.as_str()) {
            return None;
        }

        let under_cursor = ctx.node_under_cursor.as_ref()?;

        match under_cursor.kind() {
            "any_identifier"
                if ctx.matches_ancestor_history(&["relation", "object_reference"])
                    || ctx
                        .matches_ancestor_history(&["grantable_on_table", "object_reference"]) =>
            {
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

            "any_identifier"
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

            "column_identifier" => {
                if let Some(table_or_alias) = ctx.schema_or_alias_name.as_ref() {
                    Some(HoveredNode::Column(NodeIdentification::SchemaAndName((
                        table_or_alias.clone(),
                        node_content,
                    ))))
                } else {
                    Some(HoveredNode::Column(NodeIdentification::Name(node_content)))
                }
            }

            "any_identifier"
                if ctx.matches_ancestor_history(&["invocation", "object_reference"]) =>
            {
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

            "any_identifier"
                if ctx.matches_one_of_ancestors(&[
                    "alter_role",
                    "policy_to_role",
                    "role_specification",
                ]) || ctx.before_cursor_matches_kind(&["keyword_revoke"]) =>
            {
                Some(HoveredNode::Role(NodeIdentification::Name(node_content)))
            }
            "grant_role" | "policy_role" => {
                Some(HoveredNode::Role(NodeIdentification::Name(node_content)))
            }

            "any_identifier"
                if (
                    // hover over custom type in `create table` or `returns`
                    (ctx.matches_ancestor_history(&["type", "object_reference"])
                    && ctx.node_under_cursor_is_within_field_name("custom_type"))

                    // hover over type in `select` clause etc…                    
                    || (ctx
                        .matches_ancestor_history(&["field_qualifier", "object_reference"])
                        && ctx.before_cursor_matches_kind(&["("])))

                    // make sure we're not checking against an alias
                    && ctx
                        .get_mentioned_table_for_alias(
                            node_content.replace(['(', ')'], "").as_str(),
                        )
                        .is_none() =>
            {
                let sanitized = node_content.replace(['(', ')'], "");
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

            // quoted columns
            "literal" if ctx.matches_ancestor_history(&["select_expression", "term"]) => {
                Some(HoveredNode::Column(NodeIdentification::Name(node_content)))
            }

            "grant_table" => {
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

fn looks_like_sql_param(content: &str) -> bool {
    (content.starts_with("$") && !content.starts_with("$$"))
        || (content.starts_with(":") && !content.starts_with("::"))
        || (content.starts_with("@"))
        || content.starts_with("?")
}
