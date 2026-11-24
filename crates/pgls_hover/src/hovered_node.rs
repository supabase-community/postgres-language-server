use pgls_catalog::identifier_name;
use pgls_treesitter::WrappingClause;
use pgls_treesitter_grammar::{Field, Kind};

#[derive(Debug)]
pub(crate) enum HoveredNode {
    Schema(String),
    Table((Option<String>, String)),
    Function((Option<String>, String)),
    Column((Option<String>, Option<String>, String)),
    Role(String),
    PostgresType((Option<String>, String)),

    #[allow(dead_code, reason = "policies aren't found under the cursor yet")]
    Policy((Option<String>, String)),
}

impl HoveredNode {
    pub(crate) fn get(ctx: &pgls_treesitter::context::TreesitterContext) -> Option<Self> {
        let node_content = ctx.get_node_under_cursor_content()?;

        if looks_like_sql_param(node_content.as_str()) {
            return None;
        }

        // The catalog holds names, not identifiers as written.
        let name = identifier_name(&node_content);

        let under_cursor = &ctx.node_under_cursor;

        match Kind::of(under_cursor) {
            Kind::COLUMN_IDENTIFIER => Some(HoveredNode::Column((
                ctx.head_qualifier_sanitized(),
                ctx.tail_qualifier_sanitized(),
                name,
            ))),
            Kind::FUNCTION_IDENTIFIER => Some(HoveredNode::Function((
                ctx.tail_qualifier_sanitized(),
                name,
            ))),
            Kind::POLICY_IDENTIFIER => {
                Some(HoveredNode::Policy((ctx.tail_qualifier_sanitized(), name)))
            }
            Kind::TABLE_IDENTIFIER => {
                Some(HoveredNode::Table((ctx.tail_qualifier_sanitized(), name)))
            }

            Kind::SCHEMA_IDENTIFIER => Some(HoveredNode::Schema(name)),
            Kind::ROLE_IDENTIFIER => Some(HoveredNode::Role(name)),

            Kind::ANY_IDENTIFIER
                if ctx.history_ends_with(&[Kind::TABLE_REFERENCE, Kind::ANY_IDENTIFIER]) =>
            {
                Some(HoveredNode::Table((ctx.tail_qualifier_sanitized(), name)))
            }

            Kind::ANY_IDENTIFIER
                if ctx.history_ends_with(&[Kind::OBJECT_REFERENCE, Kind::ANY_IDENTIFIER])
                    && ctx.wrapping_clause_type.as_ref().is_some_and(|clause| {
                        matches!(
                            clause,
                            WrappingClause::AlterPolicy
                                | WrappingClause::CreatePolicy
                                | WrappingClause::DropPolicy
                        )
                    }) =>
            {
                Some(HoveredNode::Table((ctx.tail_qualifier_sanitized(), name)))
            }

            Kind::ANY_IDENTIFIER
                if ctx.history_ends_with(&[
                    Kind::BINARY_EXPRESSION,
                    Kind::OBJECT_REFERENCE,
                    Kind::ANY_IDENTIFIER,
                ]) || ctx.history_ends_with(&[Kind::TERM, Kind::OBJECT_REFERENCE, Kind::ANY_IDENTIFIER])
                    // The value of `SET col = t.col`, and of `MERGE ... INSERT VALUES (t.col)`,
                    // but not its qualifier `t`, which may be the name of a column too.
                    || ((ctx.history_ends_with(&[Kind::ASSIGNMENT, Kind::OBJECT_REFERENCE, Kind::ANY_IDENTIFIER])
                        || ctx.history_ends_with(&[
                            Kind::MERGE_INSERT,
                            Kind::INSERT_VALUES,
                            Kind::OBJECT_REFERENCE,
                            Kind::ANY_IDENTIFIER,
                        ]))
                        && ctx.node_under_cursor_is_within_field(&[
                            Field::OBJECT_REFERENCE_1OF1,
                            Field::OBJECT_REFERENCE_2OF2,
                            Field::OBJECT_REFERENCE_3OF3,
                        ])) =>
            {
                Some(HoveredNode::Column((
                    ctx.head_qualifier_sanitized(),
                    ctx.tail_qualifier_sanitized(),
                    name,
                )))
            }

            Kind::ANY_IDENTIFIER
                if ctx.history_ends_with(&[
                    Kind::INVOCATION,
                    Kind::FUNCTION_REFERENCE,
                    Kind::ANY_IDENTIFIER,
                ]) =>
            {
                Some(HoveredNode::Function((
                    ctx.tail_qualifier_sanitized(),
                    name,
                )))
            }

            Kind::ANY_IDENTIFIER
                if (
                    // hover over custom type in `create table` or `returns`
                    (ctx.history_ends_with(&[Kind::TYPE, Kind::OBJECT_REFERENCE, Kind::ANY_IDENTIFIER])
                    && ctx.node_under_cursor_is_within_field(&[Field::CUSTOM_TYPE]))

                    // hover over type in `select` clause etc…
                    || (ctx
                        .history_ends_with(&[Kind::FIELD_SELECTION, Kind::PARENTHESIZED_EXPRESSION, Kind::OBJECT_REFERENCE, Kind::ANY_IDENTIFIER])
                        && ctx.node_under_cursor_is_within_field(&[Field::OBJECT_REFERENCE_1OF1, Field::OBJECT_REFERENCE_2OF2])))

                    // make sure we're not checking against an alias
                    && ctx
                        .get_mentioned_table_for_alias(
                            name.as_str(),
                        )
                        .is_none() =>
            {
                Some(HoveredNode::PostgresType((
                    ctx.tail_qualifier_sanitized(),
                    name,
                )))
            }

            // quoted columns
            Kind::LITERAL
                if ctx.history_ends_with(&[Kind::SELECT_EXPRESSION, Kind::TERM, Kind::LITERAL]) =>
            {
                Some(HoveredNode::Column((
                    ctx.head_qualifier_sanitized(),
                    ctx.tail_qualifier_sanitized(),
                    name,
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
