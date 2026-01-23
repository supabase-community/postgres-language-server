use pgls_schema_cache::ProcKind;
use pgls_treesitter::{
    context::{TreesitterContext, WrappingClause, WrappingNode},
    goto_node_at_position, previous_sibling_completed,
};
use tree_sitter::{InputEdit, Point, Tree};

use super::CompletionRelevanceData;

/// Parses SQL with a keyword injected at the given position, using incremental parsing.
///
/// Temporarily edits `shared_tree` to hint tree-sitter about the change location,
/// then undoes the edit so the tree can be reused for subsequent keywords.
fn parse_with_replaced_keyword(
    shared_tree: &mut Tree,
    text: &str,
    keyword: &str,
    node: tree_sitter::Node,
) -> Tree {
    let start_position = node.start_position();
    let start_byte = node.start_byte();
    let old_end_position = node.end_position();
    let old_end_byte = node.end_byte();

    let new_end_byte = start_byte + keyword.len();
    let new_end_position = Point {
        row: start_position.row,
        column: start_position.column + keyword.len(),
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
        &text[..start_byte],
        keyword,
        &text[old_end_byte..]
    );

    let tree = parser.parse(replaced_sql, Some(shared_tree)).unwrap();

    // undo edit so shared_tree can be reused
    shared_tree.edit(&InputEdit {
        new_end_byte: old_end_byte,
        new_end_position: old_end_position,
        old_end_byte: new_end_byte,
        old_end_position: new_end_position,
        start_byte,
        start_position,
    });

    tree
}

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

    /// Filters out keywords marked with `require_prefix` unless the user has started typing them.
    ///
    /// Some keywords like `and`, `as`, `like` are too noisy to show unprompted since they'd
    /// appear in nearly every completion list. We only suggest them once the user types at
    /// least the first character.
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

        if !kw.name.starts_with(content.chars().next().unwrap()) {
            return None;
        }

        Some(())
    }

    /// Validates whether a keyword would produce valid SQL at the current cursor position.
    ///
    /// Uses speculative parsing: injects the keyword into the SQL text at the cursor position,
    /// re-parses, and checks if the result is grammatically valid. The `shared_tree` is
    /// temporarily edited to enable tree-sitter's incremental parsing optimization.
    ///
    /// # Validation rules
    ///
    /// 1. **Parse error** → reject. The keyword produces invalid SQL.
    ///
    /// 2. **Error recovery** → accept. If the previous clause was an ERROR node and injecting
    ///    the keyword fixes it, the keyword helped tree-sitter recover.
    ///
    /// 3. **Statement boundary** → only accept `starts_statement` keywords (like `SELECT`,
    ///    `INSERT`) when there's no current clause or we're after a semicolon.
    ///
    /// 4. **Clause completion check** → the core logic. Replacing/inserting a keyword is valid
    ///    only if it doesn't
    ///      - make a current finished clause unfinished
    ///      - fully replaces the previous unfinished clause
    ///
    ///    See `GRAMMAR_GUIDELINES.md` for how `@end` field markers track clause completion.
    fn valid_keyword(&self, ctx: &TreesitterContext, shared_tree: &mut Tree) -> Option<()> {
        let keyword = if let CompletionRelevanceData::Keyword(kw) = self.data {
            kw
        } else {
            return Some(());
        };

        let tree =
            parse_with_replaced_keyword(shared_tree, ctx.text, keyword.name, ctx.node_under_cursor);

        if tree.root_node().has_error() {
            return None;
        }

        if ctx.previous_clause.is_some_and(|n| n.kind() == "ERROR") {
            return Some(());
        }

        let clause_to_investigate = if ctx.node_under_cursor.kind() != "ERROR" {
            ctx.current_clause
        } else {
            ctx.previous_clause
        };

        if clause_to_investigate.is_none() || ctx.previous_clause.is_some_and(|n| n.kind() == ";") {
            if keyword.starts_statement {
                return Some(());
            } else {
                return None;
            }
        }

        let start_byte = ctx.node_under_cursor.start_byte();

        if let Some(investigated_node) = goto_node_at_position(&tree, start_byte) {
            if !previous_sibling_completed(investigated_node) {
                return None;
            }

            let old_unfinished_parent = clause_to_investigate.unwrap();

            let mut new_parent = investigated_node.parent();

            while let Some(parent) = new_parent {
                if !previous_sibling_completed(parent) {
                    return None;
                }

                if parent.kind() == old_unfinished_parent.kind()
                    || parent.start_byte() < old_unfinished_parent.start_byte()
                {
                    break;
                }

                new_parent = parent.parent();
            }

            if let Some(new_parent) = new_parent {
                if new_parent.kind() != old_unfinished_parent.kind() {
                    return None;
                }

                if new_parent.start_byte() != old_unfinished_parent.start_byte() {
                    return None;
                }

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
    #[ignore = "TODO: fix keyword completion filtering"]
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
