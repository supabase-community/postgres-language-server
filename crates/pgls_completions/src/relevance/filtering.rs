use pgls_schema_cache::ProcKind;
use pgls_treesitter::context::{TreesitterContext, WrappingClause, WrappingNode};

use super::CompletionRelevanceData;

#[derive(Debug)]
pub(crate) struct CompletionFilter<'a> {
    data: CompletionRelevanceData<'a>,
}

impl<'a> From<CompletionRelevanceData<'a>> for CompletionFilter<'a> {
    fn from(value: CompletionRelevanceData<'a>) -> Self {
        Self { data: value }
    }
}

impl CompletionFilter<'_> {
    pub fn is_relevant(&self, ctx: &TreesitterContext) -> Option<()> {
        self.completable_context(ctx)?;

        self.check_node_type(ctx)
            // we want to rely on treesitter more, so checking the clause is a fallback
            .or_else(|| self.check_clause(ctx))?;

        self.check_invocation(ctx)?;
        self.check_mentioned_schema_or_alias(ctx)?;

        Some(())
    }

    fn completable_context(&self, ctx: &TreesitterContext) -> Option<()> {
        if ctx.wrapping_node_kind.is_none() && ctx.wrapping_clause_type.is_none() {
            return None;
        }

        let current_node_kind = ctx
            .node_under_cursor
            .as_ref()
            .map(|n| n.kind())
            .unwrap_or("");

        if current_node_kind.starts_with("keyword_")
            || current_node_kind == "="
            || current_node_kind == ","
            || current_node_kind == "ERROR"
        {
            return None;
        }

        // "literal" nodes can be identfiers wrapped in quotes:
        // `select "email" from auth.users;`
        // Here, "email" is a literal node.
        if current_node_kind == "literal" {
            match self.data {
                CompletionRelevanceData::Column(_) => match ctx.wrapping_clause_type.as_ref() {
                    Some(WrappingClause::Select)
                    | Some(WrappingClause::Where)
                    | Some(WrappingClause::Join { .. })
                    | Some(WrappingClause::Update)
                    | Some(WrappingClause::Delete)
                    | Some(WrappingClause::Insert)
                    | Some(WrappingClause::DropColumn)
                    | Some(WrappingClause::AlterColumn)
                    | Some(WrappingClause::RenameColumn) => {
                        // the literal is probably a column
                    }
                    _ => return None,
                },
                _ => return None,
            }
        }

        // No autocompletions if there are two identifiers without a separator.
        if ctx.node_under_cursor.as_ref().is_some_and(|node| {
            node.prev_sibling().is_some_and(|p| {
                (p.kind() == "any_identifier" || p.kind() == "object_reference")
                    && node.kind() == "any_identifier"
            })
        }) {
            return None;
        }

        // no completions if we're right after an asterisk:
        // `select * {}`
        if ctx.node_under_cursor.as_ref().is_some_and(|node| {
            node.prev_sibling()
                .is_some_and(|p| (p.kind() == "all_fields") && node.kind() == "any_identifier")
        }) {
            return None;
        }

        Some(())
    }

    fn check_node_type(&self, ctx: &TreesitterContext) -> Option<()> {
        let kind = ctx.node_under_cursor.as_ref().map(|n| n.kind())?;

        let is_allowed = match kind {
            "column_identifier" => {
                matches!(self.data, CompletionRelevanceData::Column(_))
                    && !ctx.matches_ancestor_history(&["insert_values", "field"])
                    && !ctx.node_under_cursor_is_within_field_name("binary_expr_right")
            }
            _ => false,
        };

        if is_allowed { Some(()) } else { None }
    }

    fn check_clause(&self, ctx: &TreesitterContext) -> Option<()> {
        ctx.wrapping_clause_type
            .as_ref()
            .map(|clause| {
                match self.data {
                    CompletionRelevanceData::Table(_) => match clause {
                        WrappingClause::From | WrappingClause::Update => true,

                        WrappingClause::RevokeStatement | WrappingClause::GrantStatement => ctx
                            .matches_ancestor_history(&["grantable_on_table", "object_reference"]),

                        WrappingClause::Join { on_node: None } => true,
                        WrappingClause::Join { on_node: Some(on) } => ctx
                            .node_under_cursor
                            .as_ref()
                            .is_some_and(|cn| cn.start_byte() < on.end_byte()),

                        WrappingClause::Insert => {
                            ctx.wrapping_node_kind
                                .as_ref()
                                .is_none_or(|n| n != &WrappingNode::List)
                                && (ctx.before_cursor_matches_kind(&["keyword_into"])
                                    || (ctx.before_cursor_matches_kind(&["."])
                                        && ctx.matches_ancestor_history(&["object_reference"])))
                        }

                        WrappingClause::DropTable | WrappingClause::AlterTable => ctx
                            .before_cursor_matches_kind(&[
                                "keyword_exists",
                                "keyword_only",
                                "keyword_table",
                            ]),

                        WrappingClause::CreatePolicy
                        | WrappingClause::AlterPolicy
                        | WrappingClause::DropPolicy => {
                            ctx.matches_ancestor_history(&["object_reference"])
                                && ctx.before_cursor_matches_kind(&["keyword_on", "."])
                        }

                        _ => false,
                    },

                    CompletionRelevanceData::Column(_) => {
                        match clause {
                            WrappingClause::Select
                            | WrappingClause::Update
                            | WrappingClause::Delete
                            | WrappingClause::DropColumn => true,

                            WrappingClause::RenameColumn => ctx
                                .before_cursor_matches_kind(&["keyword_rename", "keyword_column"]),

                            WrappingClause::AlterColumn => {
                                ctx.before_cursor_matches_kind(&["keyword_alter", "keyword_column"])
                            }

                            // We can complete columns in JOIN cluases, but only if we are after the
                            // ON node in the "ON u.id = posts.user_id" part.
                            WrappingClause::Join { on_node: Some(on) } => ctx
                                .node_under_cursor
                                .as_ref()
                                .is_some_and(|cn| cn.start_byte() >= on.end_byte()),

                            // we are in a JOIN, but definitely not after an ON
                            WrappingClause::Join { on_node: None } => false,

                            WrappingClause::Insert => ctx
                                .wrapping_node_kind
                                .as_ref()
                                .is_some_and(|n| n == &WrappingNode::List),

                            // only autocomplete left side of binary expression
                            WrappingClause::Where => {
                                ctx.before_cursor_matches_kind(&["keyword_and", "keyword_where"])
                                    || (ctx.before_cursor_matches_kind(&["field_qualifier"])
                                        && ctx.matches_ancestor_history(&["field"]))
                            }

                            WrappingClause::CheckOrUsingClause => {
                                ctx.before_cursor_matches_kind(&["(", "keyword_and"])
                                    || ctx.wrapping_node_kind.as_ref().is_some_and(|nk| {
                                        matches!(nk, WrappingNode::BinaryExpression)
                                    })
                            }

                            _ => false,
                        }
                    }

                    CompletionRelevanceData::Function(f) => match clause {
                        WrappingClause::From
                        | WrappingClause::Select
                        | WrappingClause::Where
                        | WrappingClause::Join { .. } => true,

                        WrappingClause::CheckOrUsingClause => {
                            !matches!(f.kind, ProcKind::Aggregate)
                                && (ctx.before_cursor_matches_kind(&["(", "keyword_and"])
                                    || ctx.wrapping_node_kind.as_ref().is_some_and(|nk| {
                                        matches!(nk, WrappingNode::BinaryExpression)
                                    }))
                        }

                        _ => false,
                    },

                    CompletionRelevanceData::Schema(_) => match clause {
                        WrappingClause::Select
                        | WrappingClause::From
                        | WrappingClause::Join { .. }
                        | WrappingClause::Update
                        | WrappingClause::Delete => true,

                        WrappingClause::RevokeStatement | WrappingClause::GrantStatement => {
                            (ctx.matches_ancestor_history(&[
                                "grantable_on_table",
                                "object_reference",
                            ]) && ctx.schema_or_alias_name.is_none())
                                || ctx.matches_ancestor_history(&["grantable_on_all"])
                        }

                        WrappingClause::Where => {
                            ctx.before_cursor_matches_kind(&["keyword_and", "keyword_where"])
                        }

                        WrappingClause::DropTable | WrappingClause::AlterTable => ctx
                            .before_cursor_matches_kind(&[
                                "keyword_exists",
                                "keyword_only",
                                "keyword_table",
                            ]),

                        WrappingClause::Insert => {
                            ctx.wrapping_node_kind
                                .as_ref()
                                .is_none_or(|n| n != &WrappingNode::List)
                                && ctx.before_cursor_matches_kind(&["keyword_into"])
                        }

                        WrappingClause::CreatePolicy
                        | WrappingClause::AlterPolicy
                        | WrappingClause::DropPolicy => {
                            ctx.before_cursor_matches_kind(&["keyword_on"])
                        }

                        _ => false,
                    },

                    CompletionRelevanceData::Policy(_) => {
                        matches!(
                            clause,
                            // not CREATE – there can't be existing policies.
                            WrappingClause::AlterPolicy | WrappingClause::DropPolicy
                        ) && ctx.before_cursor_matches_kind(&["keyword_policy", "keyword_exists"])
                    }

                    CompletionRelevanceData::Role(_) => match clause {
                        WrappingClause::DropRole | WrappingClause::AlterRole => true,

                        WrappingClause::SetStatement => ctx
                            .before_cursor_matches_kind(&["keyword_role", "keyword_authorization"]),

                        WrappingClause::RevokeStatement | WrappingClause::GrantStatement => {
                            ctx.matches_ancestor_history(&["role_specification"])
                                || ctx.node_under_cursor.as_ref().is_some_and(|k| {
                                    k.kind() == "any_identifier"
                                        && ctx.before_cursor_matches_kind(&[
                                            "keyword_grant",
                                            "keyword_revoke",
                                            "keyword_for",
                                        ])
                                })
                        }

                        WrappingClause::AlterPolicy | WrappingClause::CreatePolicy => {
                            ctx.before_cursor_matches_kind(&["keyword_to"])
                                && ctx.matches_ancestor_history(&["policy_to_role"])
                        }

                        _ => false,
                    },
                }
            })
            .and_then(|is_ok| if is_ok { Some(()) } else { None })
    }

    fn check_invocation(&self, ctx: &TreesitterContext) -> Option<()> {
        if !ctx.is_invocation {
            return Some(());
        }

        match self.data {
            CompletionRelevanceData::Table(_) | CompletionRelevanceData::Column(_) => return None,
            _ => {}
        }

        Some(())
    }

    fn check_mentioned_schema_or_alias(&self, ctx: &TreesitterContext) -> Option<()> {
        if ctx.schema_or_alias_name.is_none() {
            return Some(());
        }

        let schema_or_alias = ctx.schema_or_alias_name.as_ref().unwrap().replace('"', "");

        let matches = match self.data {
            CompletionRelevanceData::Table(table) => table.schema == schema_or_alias,
            CompletionRelevanceData::Function(f) => f.schema == schema_or_alias,
            CompletionRelevanceData::Column(col) => ctx
                .get_mentioned_table_for_alias(&schema_or_alias)
                .is_some_and(|t| t == &col.table_name),

            // we should never allow schema suggestions if there already was one.
            CompletionRelevanceData::Schema(_) => false,
            // no policy or row completion if user typed a schema node first.
            CompletionRelevanceData::Policy(_) | CompletionRelevanceData::Role(_) => false,
        };

        if !matches {
            return None;
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::test_helper::{
        CompletionAssertion, assert_complete_results, assert_no_complete_results,
    };

    use pgls_test_utils::QueryWithCursorPosition;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completion_after_asterisk(pool: PgPool) {
        let setup = r#"
            create table users (
                id serial primary key,
                email text,
                address text
            );
        "#;

        pool.execute(setup).await.unwrap();

        assert_no_complete_results(
            format!("select * {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            None,
            &pool,
        )
        .await;

        // if there s a COMMA after the asterisk, we're good
        assert_complete_results(
            format!("select *, {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::Label("address".into()),
                CompletionAssertion::Label("email".into()),
                CompletionAssertion::Label("id".into()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completion_after_create_table(pool: PgPool) {
        assert_no_complete_results(
            format!("create table {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completion_in_column_definitions(pool: PgPool) {
        let query = format!(
            r#"create table instruments ( {} )"#,
            QueryWithCursorPosition::cursor_marker()
        );
        assert_no_complete_results(query.as_str(), None, &pool).await;
    }
}
