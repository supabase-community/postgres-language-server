use crate::{
    CompletionItemKind,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

pub fn complete_policies<'a>(ctx: &CompletionContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_policies = &ctx.schema_cache.policies;

    for pol in available_policies {
        let relevance = CompletionRelevanceData::Policy(pol);

        let item = PossibleCompletionItem {
            label: pol.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("Table: {}", pol.table_name),
            kind: CompletionItemKind::Policy,
            completion_text: None,
        };

        builder.add_item(item);
    }
}
