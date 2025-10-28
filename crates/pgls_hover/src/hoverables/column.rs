use std::fmt::Write;

use pgls_schema_cache::{Column, SchemaCache};
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

impl ToHoverMarkdown for pgls_schema_cache::Column {
    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error> {
        write!(
            writer,
            "`{}.{}.{}`",
            self.schema_name, self.table_name, self.name
        )
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(comment) = &self.comment {
            write!(writer, "Comment: '{comment}'")?;
            writeln!(writer)?;
        }

        if let Some(tname) = self.type_name.as_ref() {
            write!(writer, "{tname}")?;
            if let Some(l) = self.varchar_length {
                write!(writer, "({l})")?;
            }
        } else {
            write!(writer, "typeid: {}", self.type_id)?;
        }

        if self.is_primary_key {
            write!(writer, " - 🔑 primary key")?;
        } else if self.is_unique {
            write!(writer, " - unique")?;
        }

        if self.is_nullable {
            write!(writer, " - nullable")?;
        } else {
            write!(writer, " - not null")?;
        }

        writeln!(writer)?;

        Ok(true)
    }

    fn hover_footer<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(default) = &self.default_expr {
            writeln!(writer)?;
            write!(writer, "Default: {default}")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ContextualPriority for Column {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        // high score if we match the specific alias or table being referenced in the cursor context
        if let Some(table_or_alias) = ctx.schema_or_alias_name.as_ref() {
            if table_or_alias.replace('"', "") == self.table_name.as_str() {
                score += 250.0;
            } else if let Some(table_name) = ctx.get_mentioned_table_for_alias(table_or_alias) {
                if table_name == self.table_name.as_str() {
                    score += 250.0;
                }
            }
        }

        if ctx
            .get_mentioned_relations(&Some(self.schema_name.clone()))
            .is_some_and(|t| t.contains(&self.table_name))
        {
            score += 150.0;
        } else if ctx
            .get_mentioned_relations(&None)
            .is_some_and(|t| t.contains(&self.table_name))
        {
            score += 100.0;
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

#[cfg(test)]
mod tests {
    use pgls_test_utils::QueryWithCursorPosition;

    use crate::{
        contextual_priority::prioritize_by_context, hoverables::test_helper::create_test_context,
    };

    use super::*;

    fn create_test_column(schema: &str, table: &str, name: &str, is_pk: bool) -> Column {
        Column {
            number: 1,
            name: name.to_string(),
            table_name: table.to_string(),
            table_oid: 1,
            class_kind: pgls_schema_cache::ColumnClassKind::OrdinaryTable,
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
