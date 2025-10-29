use crate::{
    CompletionItemKind,
    builder::{CompletionBuilder, PossibleCompletionItem},
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};
use pgls_schema_cache::SchemaCache;
use pgls_treesitter::TreesitterContext;

pub fn complete_roles<'a>(
    _ctx: &TreesitterContext<'a>,
    schema_cache: &'a SchemaCache,
    builder: &mut CompletionBuilder<'a>,
) {
    let available_roles = &schema_cache.roles;

    for role in available_roles {
        let relevance = CompletionRelevanceData::Role(role);

        let item = PossibleCompletionItem {
            label: role.name.chars().take(35).collect::<String>(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: role.name.clone(),
            kind: CompletionItemKind::Role,
            completion_text: None,
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

    const SETUP: &str = r#"
            create table users (
              id serial primary key,
              email varchar,
              address text
            );
        "#;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_drop_role(pool: PgPool) {
        assert_complete_results(
            format!("drop role {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            Some(SETUP),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_alter_role(pool: PgPool) {
        assert_complete_results(
            format!("alter role {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            Some(SETUP),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_set_statement(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!("set role {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "set session authorization {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_policies(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                r#"create policy "my cool policy" on public.users
            as restrictive 
            for all
            to {}
            using (true);"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"create policy "my cool policy" on public.users
            for select
            to {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_grant_statements(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                r#"grant select
                    on table public.users
                    to {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                // recognizing already mentioned roles is not supported for now
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"grant select
                    on table public.users
                    to owner, {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                // recognizing already mentioned roles is not supported for now
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"grant {} to owner"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                // recognizing already mentioned roles is not supported for now
                CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
                CompletionAssertion::LabelAndKind(
                    "test_login".into(),
                    crate::CompletionItemKind::Role,
                ),
                CompletionAssertion::LabelAndKind(
                    "test_nologin".into(),
                    crate::CompletionItemKind::Role,
                ),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_statements(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let queries = vec![
            format!(
                "revoke {} from owner",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "revoke admin option for {} from owner",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "revoke owner from {}",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "revoke all on schema public from {} granted by",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "revoke all on schema public from owner, {}",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "revoke all on table users from owner, {}",
                QueryWithCursorPosition::cursor_marker()
            ),
        ];

        for query in queries {
            assert_complete_results(
                query.as_str(),
                vec![
                    // recognizing already mentioned roles is not supported for now
                    CompletionAssertion::LabelAndKind(
                        "owner".into(),
                        crate::CompletionItemKind::Role,
                    ),
                    CompletionAssertion::LabelAndKind(
                        "test_login".into(),
                        crate::CompletionItemKind::Role,
                    ),
                    CompletionAssertion::LabelAndKind(
                        "test_nologin".into(),
                        crate::CompletionItemKind::Role,
                    ),
                ],
                None,
                &pool,
            )
            .await;
        }
    }
}
