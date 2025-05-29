use crate::{
    CompletionItemKind,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

pub fn complete_roles<'a>(ctx: &CompletionContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_roles = &ctx.schema_cache.roles;

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
    use sqlx::PgPool;

    use crate::test_helper::{CURSOR_POS, CompletionAssertion, assert_complete_results};

    const SETUP: &'static str = r#"
            create table users (
              id serial primary key,
              email varchar,
              address text
            );
        "#;

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn works_in_drop_role(pool: PgPool) {
        assert_complete_results(
            format!("drop role {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("admin".into(), crate::CompletionItemKind::Role),
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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn works_in_alter_role(pool: PgPool) {
        assert_complete_results(
            format!("alter role {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("admin".into(), crate::CompletionItemKind::Role),
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

    async fn works_in_set_statement() {
        // set role ROLE;
        // set session authorization ROLE;
    }

    async fn works_in_policies() {}

    async fn works_in_grant_statements() {
        // grant select on my_table to ROLE;
        // grant ROLE to OTHER_ROLE with admin option;
    }

    async fn works_in_revoke_statements() {
        // revoke select on my_table from ROLE;
        // revoke ROLE from OTHER_ROLE;
        // revoke admin option for ROLE from OTHER_ROLE;
    }
}
