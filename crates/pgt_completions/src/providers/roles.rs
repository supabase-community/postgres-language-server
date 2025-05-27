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
    use crate::test_helper::{CURSOR_POS, CompletionAssertion, assert_complete_results};

    const SETUP: &'static str = r#"
            do $$
            begin
                if not exists (
                    select from pg_catalog.pg_roles
                    where rolname = 'test'
                ) then
                    create role test;
                end if;
            end $$;

            create table users (
              id serial primary key,
              email varchar,
              address text
            );
        "#;

    #[tokio::test]
    async fn works_in_drop_role() {
        assert_complete_results(
            format!("drop role {}", CURSOR_POS).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "test".into(),
                crate::CompletionItemKind::Role,
            )],
            SETUP,
        )
        .await;
    }

    #[tokio::test]
    async fn works_in_alter_role() {
        assert_complete_results(
            format!("alter role {}", CURSOR_POS).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "test".into(),
                crate::CompletionItemKind::Role,
            )],
            SETUP,
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
