use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::get_range_to_replace;

pub fn complete_policies<'a>(ctx: &CompletionContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_policies = &ctx.schema_cache.policies;

    for pol in available_policies {
        let relevance = CompletionRelevanceData::Policy(pol);

        let item = PossibleCompletionItem {
            label: pol.name.chars().take(35).collect::<String>(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("{}", pol.table_name),
            kind: CompletionItemKind::Policy,
            completion_text: Some(CompletionText {
                text: format!("\"{}\"", pol.name),
                range: get_range_to_replace(ctx),
            }),
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{CURSOR_POS, CompletionAssertion, assert_complete_results};

    #[tokio::test]
    async fn completes_within_quotation_marks() {
        let setup = r#"
            create table users (
                id serial primary key,
                email text
            );

            create policy "should never have access" on users
                as restrictive
                for all
                to public
                using (false);
        "#;

        assert_complete_results(
            format!("alter policy \"{}\" on users;", CURSOR_POS).as_str(),
            vec![CompletionAssertion::Label(
                "should never have access".into(),
            )],
            setup,
        )
        .await;
    }
}
