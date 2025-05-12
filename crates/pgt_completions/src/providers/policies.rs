use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::get_range_to_replace;

pub fn complete_policies<'a>(ctx: &CompletionContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_policies = &ctx.schema_cache.policies;

    let has_quotes = ctx
        .get_node_under_cursor_content()
        .is_some_and(|c| c.starts_with('"') && c.ends_with('"'));

    for pol in available_policies {
        let relevance = CompletionRelevanceData::Policy(pol);

        let item = PossibleCompletionItem {
            label: pol.name.chars().take(35).collect::<String>(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("{}", pol.table_name),
            kind: CompletionItemKind::Policy,
            completion_text: if !has_quotes {
                Some(CompletionText {
                    text: format!("\"{}\"", pol.name),
                    range: get_range_to_replace(ctx),
                })
            } else {
                None
            },
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        complete,
        test_helper::{
            CURSOR_POS, CompletionAssertion, assert_complete_results, get_test_params,
            test_against_connection_string,
        },
    };

    #[tokio::test]
    async fn completes_within_quotation_marks() {
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

        assert_complete_results(
            format!("alter policy \"{}\" on private.users;", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::Label("read for public users disallowed".into()),
                CompletionAssertion::Label("write for public users allowed".into()),
            ],
            setup,
        )
        .await;

        assert_complete_results(
            format!("alter policy \"w{}\" on private.users;", CURSOR_POS).as_str(),
            vec![CompletionAssertion::Label(
                "write for public users allowed".into(),
            )],
            setup,
        )
        .await;
    }

    #[tokio::test]
    async fn sb_test() {
        let input = format!("alter policy \"u{}\" on public.fcm_tokens;", CURSOR_POS);

        let (tree, cache) = test_against_connection_string(
            "postgresql://postgres:postgres@127.0.0.1:54322/postgres",
            input.as_str().into(),
        )
        .await;

        let result = complete(get_test_params(&tree, &cache, input.as_str().into()));

        println!("{:#?}", result);
    }
}
