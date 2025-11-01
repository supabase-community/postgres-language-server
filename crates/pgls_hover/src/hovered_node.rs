use pgls_treesitter::WrappingClause;

type NodeIdentification = (Option<String>, String);

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum HoveredNode {
    Schema(String),
    Table(NodeIdentification),
    Function(NodeIdentification),
    Column((Option<String>, Option<String>, String)),
    Policy(NodeIdentification),
    Trigger(NodeIdentification),
    Role(String),
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
            "column_identifier" => Some(HoveredNode::Column((
                ctx.identifier_qualifiers.0.clone(),
                ctx.identifier_qualifiers.1.clone(),
                node_content,
            ))),
            "function_identifier" => Some(HoveredNode::Function((
                ctx.identifier_qualifiers.1.clone(),
                node_content,
            ))),
            "policy_identifier" => Some(HoveredNode::Policy((
                ctx.identifier_qualifiers.1.clone(),
                node_content,
            ))),
            "table_identifier" => Some(HoveredNode::Table((
                ctx.identifier_qualifiers.1.clone(),
                node_content,
            ))),

            "schema_identifier" => Some(HoveredNode::Schema(node_content)),
            "role_identifier" => Some(HoveredNode::Role(node_content)),

            "any_identifier"
                if ctx.matches_ancestor_history(&["table_reference"])
                    || ctx
                        .matches_ancestor_history(&["grantable_on_table", "object_reference"]) =>
            {
                let num_sibs = ctx.num_siblings();
                if ctx.node_under_cursor_is_nth_child(1) && num_sibs > 0 {
                    return Some(HoveredNode::Schema(node_content));
                }

                Some(HoveredNode::Table((
                    ctx.identifier_qualifiers.1.clone(),
                    node_content,
                )))
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
                Some(HoveredNode::Table((
                    ctx.identifier_qualifiers.1.clone(),
                    node_content,
                )))
            }

            "any_identifier"
                if ctx.matches_ancestor_history(&["binary_expression", "object_reference"])
                    || ctx.matches_ancestor_history(&["term", "object_reference"]) =>
            {
                Some(HoveredNode::Column((
                    ctx.identifier_qualifiers.0.clone(),
                    ctx.identifier_qualifiers.1.clone(),
                    node_content,
                )))
            }

            "any_identifier"
                if ctx.matches_ancestor_history(&["invocation", "function_reference"]) =>
            {
                Some(HoveredNode::Function((
                    ctx.identifier_qualifiers.1.clone(),
                    node_content,
                )))
            }

            "any_identifier"
                if ctx.matches_one_of_ancestors(&[
                    "alter_role",
                    "policy_to_role",
                    "role_specification",
                ]) || ctx.before_cursor_matches_kind(&["keyword_revoke"]) =>
            {
                Some(HoveredNode::Role(node_content))
            }

            "any_identifier"
                if (
                    // hover over custom type in `create table` or `returns`
                    (ctx.matches_ancestor_history(&["type", "object_reference"])
                    && ctx.node_under_cursor_is_within_field_name(&["custom_type"]))

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

                Some(HoveredNode::PostgresType((
                    ctx.identifier_qualifiers.1.clone(),
                    sanitized,
                )))
            }

            // quoted columns
            "literal" if ctx.matches_ancestor_history(&["select_expression", "term"]) => {
                Some(HoveredNode::Column((
                    ctx.identifier_qualifiers.0.clone(),
                    ctx.identifier_qualifiers.1.clone(),
                    node_content,
                )))
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
