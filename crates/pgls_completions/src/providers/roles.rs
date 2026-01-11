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

    use crate::test_helper::{
        CompletionAssertion, TestCompletionsCase, TestCompletionsSuite, assert_complete_results,
    };

    use pgls_test_utils::QueryWithCursorPosition;

    const SETUP: &str = r#"
            create table users (
              id serial primary key,
              email varchar,
              address text
            );
        "#;

    fn expected_roles() -> Vec<CompletionAssertion> {
        vec![
            CompletionAssertion::LabelAndKind("anon".into(), crate::CompletionItemKind::Role),
            CompletionAssertion::LabelAndKind(
                "authenticated".into(),
                crate::CompletionItemKind::Role,
            ),
            CompletionAssertion::LabelAndKind("owner".into(), crate::CompletionItemKind::Role),
            CompletionAssertion::LabelAndKind(
                "service_role".into(),
                crate::CompletionItemKind::Role,
            ),
            CompletionAssertion::LabelAndKind("test_login".into(), crate::CompletionItemKind::Role),
            CompletionAssertion::LabelAndKind(
                "test_nologin".into(),
                crate::CompletionItemKind::Role,
            ),
        ]
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_drop_role(pool: PgPool) {
        assert_complete_results(
            format!("drop role {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            expected_roles(),
            Some(SETUP),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_alter_role(pool: PgPool) {
        assert_complete_results(
            format!("alter role {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            expected_roles(),
            Some(SETUP),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_set_role_statement(pool: PgPool) {
        TestCompletionsSuite::new(&pool, Some(SETUP))
            .with_case(TestCompletionsCase::new().type_sql("set role anon;"))
            .snapshot("works_in_set_role_statement")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_session_authorization_statement(pool: PgPool) {
        TestCompletionsSuite::new(&pool, Some(SETUP))
            .with_case(
                TestCompletionsCase::new().type_sql("set session authorization authenticated;"),
            )
            .snapshot("works_in_session_authorization_statement")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_policy_restrictive(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let mut expected = vec![
            CompletionAssertion::LabelAndKind(
                "current_role".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind(
                "current_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind("public".into(), crate::CompletionItemKind::Keyword),
            CompletionAssertion::LabelAndKind(
                "session_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
        ];
        expected.append(&mut expected_roles());

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
            expected,
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_policy_select(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let mut expected = vec![
            CompletionAssertion::LabelAndKind(
                "current_role".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind(
                "current_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind("public".into(), crate::CompletionItemKind::Keyword),
            CompletionAssertion::LabelAndKind(
                "session_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
        ];
        expected.append(&mut expected_roles());

        assert_complete_results(
            format!(
                r#"create policy "my cool policy" on public.users
            for select
            to {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected,
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_grant_on_table_to(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let mut expected = vec![
            CompletionAssertion::LabelAndKind(
                "current_role".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind(
                "current_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind("public".into(), crate::CompletionItemKind::Keyword),
            CompletionAssertion::LabelAndKind(
                "session_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
        ];
        expected.append(&mut expected_roles());

        assert_complete_results(
            format!(
                r#"grant select
                    on table public.users
                    to {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected,
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_grant_on_table_to_multiple(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let mut expected = vec![
            CompletionAssertion::LabelAndKind(
                "current_role".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind(
                "current_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind("public".into(), crate::CompletionItemKind::Keyword),
            CompletionAssertion::LabelAndKind(
                "session_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
        ];
        expected.append(&mut expected_roles());

        assert_complete_results(
            format!(
                r#"grant select
                    on table public.users
                    to owner, {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected,
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_grant_role_to(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                r#"grant {} to owner"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected_roles(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_role(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                "revoke {} from owner",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected_roles(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_admin_option_for(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                "revoke admin option for {} from owner",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected_roles(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_from(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let mut expected = vec![
            CompletionAssertion::LabelAndKind(
                "current_role".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind(
                "current_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind("public".into(), crate::CompletionItemKind::Keyword),
            CompletionAssertion::LabelAndKind(
                "session_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
        ];
        expected.append(&mut expected_roles());

        assert_complete_results(
            format!(
                "revoke owner from {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected,
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_all_on_schema_from_granted_by(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                "revoke all on schema public from {} granted by",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected_roles(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_all_on_schema_from_multiple(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        assert_complete_results(
            format!(
                "revoke all on schema public from owner, {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected_roles(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn works_in_revoke_all_on_table_from_multiple(pool: PgPool) {
        pool.execute(SETUP).await.unwrap();

        let mut expected = vec![
            CompletionAssertion::LabelAndKind(
                "current_role".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind(
                "current_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
            CompletionAssertion::LabelAndKind("public".into(), crate::CompletionItemKind::Keyword),
            CompletionAssertion::LabelAndKind(
                "session_user".into(),
                crate::CompletionItemKind::Keyword,
            ),
        ];
        expected.append(&mut expected_roles());

        assert_complete_results(
            format!(
                "revoke all on table users from owner, {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            expected,
            None,
            &pool,
        )
        .await;
    }
}
