use pgt_schema_cache::{Column, Function, Table};
use pgt_treesitter::context::TreesitterContext;

pub(crate) trait ContextualPriority {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32;
}

impl ContextualPriority for Column {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        // high score if we match the specific alias or table being referenced in the cursor context
        if let Some(table_or_alias) = ctx.schema_or_alias_name.as_ref() {
            if table_or_alias == self.table_name.as_str() {
                score += 250.0;
            } else if let Some(table_name) = ctx.mentioned_table_aliases.get(table_or_alias) {
                if table_name == self.table_name.as_str() {
                    score += 250.0;
                }
            }
        }

        // medium score if the current column maps to any of the query's mentioned
        // "(schema.)table" combinations
        for (schema_opt, tables) in &ctx.mentioned_relations {
            if tables.contains(&self.table_name) {
                if schema_opt.as_deref() == Some(&self.schema_name) {
                    score += 150.0;
                } else {
                    score += 100.0;
                }
            }
        }

        if self.schema_name == "public" && score == 0.0 {
            score += 10.0;
        }

        if self.is_primary_key && score == 0.0 {
            score += 5.0;
        }

        score
    }
}

impl ContextualPriority for Table {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        for (schema_opt, tables) in &ctx.mentioned_relations {
            if tables.contains(&self.name) {
                if schema_opt.as_deref() == Some(&self.schema) {
                    score += 200.0;
                } else {
                    score += 150.0;
                }
            }
        }

        if ctx
            .mentioned_relations
            .keys()
            .any(|schema| schema.as_deref() == Some(&self.schema))
        {
            score += 50.0;
        }

        if self.schema == "public" && score == 0.0 {
            score += 10.0;
        }

        score
    }
}

impl ContextualPriority for Function {
    fn relevance_score(&self, _ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        // built-in functions get higher priority
        if self.language == "internal" {
            score += 100.0;
        }

        // public schema functions get base priority
        if self.schema == "public" {
            score += 50.0;
        } else {
            score += 25.0;
        }

        // aggregate and window functions are commonly used
        match self.kind {
            pgt_schema_cache::ProcKind::Aggregate => score += 20.0,
            pgt_schema_cache::ProcKind::Window => score += 15.0,
            pgt_schema_cache::ProcKind::Function => score += 10.0,
            pgt_schema_cache::ProcKind::Procedure => score += 5.0,
        }

        score
    }
}

/// Will first sort the items by a score and then filter out items with a score gap algorithm.  
///
/// `[200, 180, 150, 140]` => all items are returned
///
/// `[200, 180, 15, 10]` => first two items are returned
///
/// `[200, 30, 20, 10]` => only first item is returned
pub(crate) fn prioritize_by_context<T: ContextualPriority + std::fmt::Debug>(
    items: Vec<T>,
    ctx: &TreesitterContext,
) -> Vec<T> {
    let mut scored: Vec<_> = items
        .into_iter()
        .map(|item| {
            let score = item.relevance_score(ctx);
            (item, score)
        })
        .collect();

    scored.sort_by(|(_, score_a), (_, score_b)| score_b.partial_cmp(score_a).unwrap());

    if scored.is_empty() {
        return vec![];
    }

    // always include the top result
    let top_result = scored.remove(0);
    let mut results = vec![top_result.0];
    let mut prev_score = top_result.1;

    // include additional results until we hit a significant score gap of 30%
    for (item, score) in scored.into_iter() {
        let gap = prev_score - score;
        if gap > prev_score * 0.3 {
            break;
        }
        results.push(item);
        prev_score = score;
    }

    results
}

#[cfg(test)]
mod tests {
    use pgt_test_utils::QueryWithCursorPosition;
    use pgt_text_size::TextSize;

    use super::*;

    fn create_test_column(schema: &str, table: &str, name: &str, is_pk: bool) -> Column {
        Column {
            name: name.to_string(),
            table_name: table.to_string(),
            table_oid: 1,
            class_kind: pgt_schema_cache::ColumnClassKind::OrdinaryTable,
            schema_name: schema.to_string(),
            type_id: 23,
            type_name: Some("integer".to_string()),
            is_nullable: false,
            is_primary_key: is_pk,
            is_unique: is_pk,
            default_expr: None,
            varchar_length: None,
            comment: None,
        }
    }

    fn create_test_context(query: QueryWithCursorPosition) -> TreesitterContext<'static> {
        use pgt_treesitter::TreeSitterContextParams;

        let (pos, sql) = query.get_text_and_position();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_sql::language()).unwrap();
        let tree = parser.parse(sql.clone(), None).unwrap();

        // Leak some stuff so test setup is easier
        let leaked_tree: &'static tree_sitter::Tree = Box::leak(Box::new(tree));
        let leaked_sql: &'static String = Box::leak(Box::new(sql));

        let position = TextSize::new(pos.try_into().unwrap());

        let ctx = pgt_treesitter::context::TreesitterContext::new(TreeSitterContextParams {
            position,
            text: leaked_sql,
            tree: leaked_tree,
        });

        ctx
    }

    #[test]
    fn column_scoring_prioritizes_mentioned_tables() {
        let query = format!(
            "select id{} from auth.users",
            QueryWithCursorPosition::cursor_marker()
        );

        let ctx = create_test_context(query.into());

        let auth_users_id = create_test_column("auth", "users", "id", true);
        let public_posts_id = create_test_column("public", "posts", "id", false);
        let public_users_id = create_test_column("public", "users", "id", false);

        let columns = vec![auth_users_id, public_posts_id, public_users_id];
        let result = prioritize_by_context(columns, &ctx);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].table_name, "users");
        assert_eq!(result[0].schema_name, "auth");
        assert!(result[0].is_primary_key);
    }

    #[test]
    fn column_scoring_prioritizes_mentioned_table_names() {
        let query = format!(
            "select users.id{} from ",
            QueryWithCursorPosition::cursor_marker()
        );

        let ctx = create_test_context(query.into());

        let videos_id = create_test_column("public", "videos", "id", false);
        let posts_id = create_test_column("public", "posts", "id", false);
        let users_id = create_test_column("public", "users", "id", false);

        let columns = vec![videos_id, posts_id, users_id];
        let result = prioritize_by_context(columns, &ctx);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].table_name, "users");
    }

    #[test]
    fn column_scoring_prioritizes_mentioned_tables_with_aliases() {
        let query = format!(
            "select p.id{} as post_id from auth.users u join public.posts p on u.id = p.user_id;",
            QueryWithCursorPosition::cursor_marker()
        );

        let ctx = create_test_context(query.into());

        let auth_users_id = create_test_column("auth", "users", "id", true);
        let public_posts_id = create_test_column("public", "posts", "id", false);
        let public_users_id = create_test_column("public", "users", "id", false);

        let columns = vec![auth_users_id, public_posts_id, public_users_id];
        let result = prioritize_by_context(columns, &ctx);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].table_name, "posts");
        assert_eq!(result[0].schema_name, "public");
        assert!(!result[0].is_primary_key);
    }
}
