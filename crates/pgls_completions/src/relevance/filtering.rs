use pgls_schema_cache::ProcKind;
use pgls_treesitter::{
    context::{TreesitterContext, WrappingClause, WrappingNode},
    goto_closest_parent_clause, goto_closest_parent_clause_with_multiple_children,
    goto_node_at_position,
};
use tree_sitter::{InputEdit, Point, Tree};

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
    pub fn is_relevant(&self, ctx: &TreesitterContext, shared_tree: &mut Tree) -> Option<()> {
        if let CompletionRelevanceData::Keyword(kw) = self.data {
            self.check_keyword_requires_prefix(ctx, kw)?;
            self.valid_keyword(ctx, shared_tree)?;
            Some(())
        } else {
            self.completable_context(ctx)?;

            self.check_specific_node_type(ctx)
                // we want to rely on treesitter more, so checking the clause is a fallback
                .or_else(|| self.check_clause(ctx))?;

            self.check_invocation(ctx)?;
            self.check_mentioned_schema_or_alias(ctx)?;

            Some(())
        }
    }

    fn completable_context(&self, ctx: &TreesitterContext) -> Option<()> {
        if ctx.wrapping_node_kind.is_none() && ctx.wrapping_clause_type.is_none() {
            return None;
        }

        let current_node_kind = ctx.node_under_cursor.kind();

        if current_node_kind.starts_with("keyword_")
            || current_node_kind == "="
            || current_node_kind == ","
            || current_node_kind == "ERROR"
        {
            return None;
        }

        if ctx.before_cursor_matches_kind(&["ERROR"]) {
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

        if ctx.node_under_cursor.kind() == "any_identifier"
            && ctx.history_ends_with(&["alias", "any_identifier"])
        {
            return None;
        }

        // No autocompletions if there are two identifiers without a separator.
        if ctx.node_under_cursor.prev_sibling().is_some_and(|p| {
            (p.kind() == "any_identifier" || p.kind() == "object_reference")
                && ctx.node_under_cursor.kind() == "any_identifier"
        }) {
            return None;
        }

        // no completions if we're right after an asterisk:
        // `select * {}`
        if ctx.node_under_cursor.prev_sibling().is_some_and(|p| {
            (p.kind() == "all_fields") && ctx.node_under_cursor.kind() == "any_identifier"
        }) {
            return None;
        }

        Some(())
    }

    fn check_specific_node_type(&self, ctx: &TreesitterContext) -> Option<()> {
        let kind = ctx.node_under_cursor.kind();

        let is_allowed = match kind {
            "column_identifier" => matches!(self.data, CompletionRelevanceData::Column(_)),
            "role_identifier" => matches!(self.data, CompletionRelevanceData::Role(_)),
            "function_identifier" => matches!(self.data, CompletionRelevanceData::Function(_)),
            "schema_identifier" => matches!(self.data, CompletionRelevanceData::Schema(_)),
            "table_identifier" => matches!(self.data, CompletionRelevanceData::Table(_)),
            "policy_identifier" => matches!(self.data, CompletionRelevanceData::Policy(_)),

            "any_identifier" => match self.data {
                CompletionRelevanceData::Column(_) => {
                    let matches_field = ctx.node_under_cursor_is_within_field(&[
                        "object_reference_1of1",
                        "object_reference_2of2",
                        "object_reference_3of3",
                        "column_reference_1of1",
                        "column_reference_2of2",
                        "column_reference_3of3",
                    ]);

                    let has_any_qualifier = ctx.has_any_qualifier();

                    matches_field || has_any_qualifier
                }

                CompletionRelevanceData::Schema(_) => ctx.node_under_cursor_is_within_field(&[
                    "object_reference_1of1",
                    "object_reference_1of2",
                    "object_reference_1of3",
                    "type_reference_1of1",
                    "table_reference_1of1",
                    "column_reference_1of1",
                    "column_reference_1of2",
                    "function_reference_1of1",
                ]),

                CompletionRelevanceData::Function(f) => {
                    ctx.node_under_cursor_is_within_field(&[
                        "object_reference_1of1",
                        "object_reference_2of2",
                        "function_reference_1of1",
                    ]) && !(ctx.history_ends_with(&[
                        "check_or_using_clause",
                        "binary_expression",
                        "object_reference",
                        "any_identifier",
                    ]) && matches!(f.kind, ProcKind::Aggregate))
                }

                CompletionRelevanceData::Table(_) => {
                    ctx.node_under_cursor_is_within_field(&[
                        "object_reference_1of1",
                        "object_reference_1of2",
                        "object_reference_2of2",
                        "object_reference_2of3",
                        "table_reference_1of1",
                        "column_reference_1of1",
                        "column_reference_1of2",
                        "column_reference_2of2",
                    ]) && !ctx.history_ends_with(&[
                        "update",
                        "assignment",
                        "column_reference",
                        "any_identifier",
                    ])
                }
                _ => false,
            },

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
                        WrappingClause::From => true,

                        WrappingClause::Update => ctx
                            .wrapping_node_kind
                            .as_ref()
                            .is_none_or(|n| n != &WrappingNode::Assignment),

                        WrappingClause::RevokeStatement | WrappingClause::GrantStatement => ctx
                            .history_ends_with(&[
                                "grantable_on_table",
                                "object_reference",
                                "any_identifier",
                            ]),

                        WrappingClause::Join { on_node: None } => true,
                        WrappingClause::Join { on_node: Some(on) } => {
                            ctx.node_under_cursor.start_byte() < on.end_byte()
                        }

                        WrappingClause::Insert => {
                            ctx.wrapping_node_kind
                                .as_ref()
                                .is_none_or(|n| n != &WrappingNode::List)
                                && (ctx.before_cursor_matches_kind(&["keyword_into"])
                                    || (ctx.before_cursor_matches_kind(&["."])
                                        && ctx.history_ends_with(&[
                                            "object_reference",
                                            "any_identifier",
                                        ])))
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
                            ctx.history_ends_with(&["object_reference", "any_identifier"])
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
                            WrappingClause::Join { on_node: Some(on) } => {
                                ctx.node_under_cursor.start_byte() >= on.end_byte()
                            }

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
                                        && ctx.history_ends_with(&["field", "any_identifier"]))
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
                        | WrappingClause::Join { .. }
                        | WrappingClause::Update
                        | WrappingClause::Delete => true,

                        WrappingClause::RevokeStatement | WrappingClause::GrantStatement => {
                            (ctx.history_ends_with(&[
                                "grantable_on_table",
                                "object_reference",
                                "any_identifier",
                            ]) && !ctx.has_any_qualifier())
                                || ctx.history_ends_with(&["grantable_on_all", "any_identifier"])
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
                            ctx.history_ends_with(&["role_specification", "any_identifier"])
                                || (ctx.node_under_cursor.kind() == "any_identifier"
                                    && ctx.before_cursor_matches_kind(&[
                                        "keyword_grant",
                                        "keyword_revoke",
                                        "keyword_for",
                                    ]))
                        }

                        WrappingClause::AlterPolicy | WrappingClause::CreatePolicy => {
                            ctx.before_cursor_matches_kind(&["keyword_to"])
                                && ctx.history_ends_with(&["policy_to_role", "any_identifier"])
                        }

                        _ => false,
                    },

                    CompletionRelevanceData::Keyword(_) => true,
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
        let tail_qualifier = match ctx.tail_qualifier_sanitized() {
            Some(q) => q,
            None => return Some(()), // no qualifier = this check passes
        };

        let matches = match self.data {
            CompletionRelevanceData::Table(table) => table.schema == tail_qualifier,
            CompletionRelevanceData::Function(f) => f.schema == tail_qualifier,
            CompletionRelevanceData::Column(col) => {
                let table = ctx
                    .get_mentioned_table_for_alias(&tail_qualifier)
                    .unwrap_or(&tail_qualifier);

                col.table_name == table.as_str()
                    && ctx
                        .head_qualifier_sanitized()
                        .is_none_or(|schema| col.schema_name == schema.as_str())
            }

            // we should never allow schema suggestions if there already was one.
            CompletionRelevanceData::Schema(_) => false,
            // no policy or row completion if user typed a schema node first.
            CompletionRelevanceData::Policy(_) | CompletionRelevanceData::Role(_) => false,
            CompletionRelevanceData::Keyword(_) => false,
        };

        if !matches {
            return None;
        }

        Some(())
    }

    fn check_keyword_requires_prefix(
        &self,
        ctx: &TreesitterContext,
        kw: &crate::providers::SqlKeyword,
    ) -> Option<()> {
        if !kw.require_prefix {
            return Some(());
        }

        let content = ctx.get_node_under_cursor_content()?;
        if content.is_empty() || crate::sanitization::is_sanitized_token(&content) {
            return None;
        }

        Some(())
    }

    fn valid_keyword(&self, ctx: &TreesitterContext, shared_tree: &mut Tree) -> Option<()> {
        let keyword = if let CompletionRelevanceData::Keyword(kw) = self.data {
            kw
        } else {
            return Some(());
        };

        let kw_name = keyword.name;

        let start_position = ctx.node_under_cursor.start_position();
        let start_byte = ctx.node_under_cursor.start_byte();
        let old_end_position = ctx.node_under_cursor.end_position();
        let old_end_byte = ctx.node_under_cursor.end_byte();

        let new_end_byte = start_byte + kw_name.len();
        let new_end_position = Point {
            row: start_position.row,
            column: start_position.column + kw_name.len(),
        };

        shared_tree.edit(&InputEdit {
            new_end_byte,
            new_end_position,
            old_end_byte,
            old_end_position,
            start_byte,
            start_position,
        });

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let replaced_sql = format!(
            "{}{}{}",
            &ctx.text[..start_byte],
            kw_name,
            &ctx.text[old_end_byte..]
        );

        let tree = parser.parse(replaced_sql, Some(shared_tree)).unwrap();

        // undo changes to shared tree
        shared_tree.edit(&InputEdit {
            new_end_byte: old_end_byte,
            new_end_position: old_end_position,
            old_end_byte: new_end_byte,
            old_end_position: new_end_position,
            start_byte,
            start_position,
        });

        if tree.root_node().has_error() {
            return None;
        } else if ctx.previous_clause.is_some_and(|n| n.kind() == "ERROR") {
            // if the previous clause has an error and injecting the keyword fixes it,
            // the keyword helped treesitter recover -> suggestable keyword
            return Some(());
        }

        let (clause_to_investigate, clause_completed) = if ctx.node_under_cursor.kind() != "ERROR" {
            (ctx.current_clause, ctx.current_clause_completed)
        } else {
            (ctx.previous_clause, ctx.previous_clause_completed)
        };

        if clause_to_investigate.is_none() {
            if keyword.starts_statement {
                return Some(());
            } else {
                return None;
            }
        }

        if clause_completed {
            return Some(());
        }

        // we allow those nodes that do not change the clause
        // e.g. `select * from table order |`; allow "by" since we remain in `order_by` clause.
        if let Some(current_node) = goto_node_at_position(&tree, start_byte) {
            if let Some(current_parent) = goto_closest_parent_clause(current_node) {
                let parent_same_kind =
                    clause_to_investigate.is_some_and(|c| c.kind() == current_parent.kind());

                let parent_same_start = clause_to_investigate
                    .is_some_and(|c| c.start_byte() == current_parent.start_byte());

                if parent_same_kind && parent_same_start {
                    return Some(());
                }
            }
        }

        /*
         * Will allow those nodes that fully exchange the parent clause BUT do not leave
         * the previous clause unfinished.
         *
         *
         * Example 1, valid replacement, `select * f|`:
         * select
         *  keyword_select
         *  select_expr
         *    term (@end)
         *      all_fields '*'
         *    alias
         *      any_identifier 'f' @end
         *
         * The `term` ends the select_expr, so it's fine to replace it with `select * from|`:
         * select
         *  keyword_select
         *  select_expr
         *    term (@end)
         *      all_fields '*'
         * from
         *   keyword_from
         *
         * -> the select_expr still has an @end!
         *
         *
         * Example 2, invalid replacement, `select f|`:
         * select
         *  keyword_select
         *  select_expr (@end)
         *    term
         *      any_identifier 'f'
         *
         * The `select_expr` ends the `select`, so it's not fine to replace it with `select from|`:
         * select
         *  keyword_select
         * from
         *  keyword_from
         *
         * The select hasn't ended, we have invalid grammar.
         *
         */
        if let Some(current_node) = goto_node_at_position(&tree, start_byte) {
            let full_exchange =
                // replacing the start byte means full exchanging
                clause_to_investigate.is_some_and(|n| n.start_byte() == current_node.start_byte());

            let leaves_clause_unfinished =
                goto_closest_parent_clause_with_multiple_children(ctx.node_under_cursor)
                    .is_some_and(|n| {
                        n.child_by_field_name("end").is_some_and(|end| {
                            end.start_byte() == ctx.node_under_cursor.start_byte()
                        })
                    });

            if full_exchange && !leaves_clause_unfinished {
                return Some(());
            }
        }

        None
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

        assert_complete_results(
            format!("select * {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "from".into(),
                crate::CompletionItemKind::Keyword,
            )],
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
