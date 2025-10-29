use pgls_schema_cache::SchemaCache;
use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::node_text_surrounded_by_quotes,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::get_range_to_replace;

pub fn complete_policies<'a>(
    ctx: &TreesitterContext<'a>,
    schema_cache: &'a SchemaCache,
    builder: &mut CompletionBuilder<'a>,
) {
    let available_policies = &schema_cache.policies;

    for pol in available_policies {
        let text = if node_text_surrounded_by_quotes(ctx) {
            // If we're within quotes, we want to change the content
            // *within* the quotes.
            pol.name.to_string()
        } else {
            format!("\"{}\"", pol.name)
        };

        let relevance = CompletionRelevanceData::Policy(pol);

        let range = get_range_to_replace(ctx);

        let item = PossibleCompletionItem {
            label: pol.name.chars().take(35).collect::<String>(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: pol.table_name.to_string(),
            kind: CompletionItemKind::Policy,
            completion_text: Some(CompletionText {
                text,
                range,
                is_snippet: false,
            }),
            detail: None,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::test_helper::{CompletionAssertion, assert_complete_results};
    use pgls_test_utils::QueryWithCursorPosition;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_within_quotation_marks(pool: PgPool) {
        let setup = r#"
            create schema private;

            create table private.users (
                id serial primary key,
                email text
            );

            create policy "read for public users disallowed" on private.users
                as restrictive
                for select
                to public
                using (false);

            create policy "write for public users allowed" on private.users
                as restrictive
                for insert
                to public
                with check (true);
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!(
                "alter policy \"{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("read for public users disallowed".into()),
                CompletionAssertion::Label("write for public users allowed".into()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "alter policy \"{}\" on private.users;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("read for public users disallowed".into()),
                CompletionAssertion::Label("write for public users allowed".into()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "alter policy \"w{}\" on private.users;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::Label(
                "write for public users allowed".into(),
            )],
            None,
            &pool,
        )
        .await;
    }
}
