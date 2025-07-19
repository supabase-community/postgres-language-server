use pgt_text_size::{TextRange, TextSize};
use pgt_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::get_range_to_replace;

pub fn complete_policies<'a>(ctx: &TreesitterContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_policies = &ctx.schema_cache.policies;

    let surrounded_by_quotes = ctx
        .get_node_under_cursor_content()
        .is_some_and(|c| c.starts_with('"') && c.ends_with('"') && c != "\"\"");

    for pol in available_policies {
        let completion_text = if surrounded_by_quotes {
            // If we're within quotes, we want to change the content
            // *within* the quotes.
            // If we attempt to replace outside the quotes, the VSCode
            // client won't show the suggestions.
            let range = get_range_to_replace(ctx);
            Some(CompletionText {
                text: pol.name.clone(),
                is_snippet: false,
                range: TextRange::new(
                    range.start() + TextSize::new(1),
                    range.end() - TextSize::new(1),
                ),
            })
        } else {
            // If we aren't within quotes, we want to complete the
            // full policy including quotation marks.
            Some(CompletionText {
                is_snippet: false,
                text: format!("\"{}\"", pol.name),
                range: get_range_to_replace(ctx),
            })
        };

        let relevance = CompletionRelevanceData::Policy(pol);

        let item = PossibleCompletionItem {
            label: pol.name.chars().take(35).collect::<String>(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: pol.table_name.to_string(),
            kind: CompletionItemKind::Policy,
            completion_text,
            detail: None,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::test_helper::{CompletionAssertion, assert_complete_results};
    use pgt_test_utils::QueryWithCursorPosition;

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
