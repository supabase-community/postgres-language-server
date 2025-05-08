use crate::{
    CompletionItemKind,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::{CompletionContext, WrappingClause},
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::{find_matching_alias_for_table, get_completion_text_with_schema_or_alias};

pub fn complete_policies<'a>(ctx: &CompletionContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_policies = &ctx.schema_cache.policies;

    for pol in available_policies {
        let relevance = CompletionRelevanceData::Policy(pol);

        let mut item = PossibleCompletionItem {
            label: pol.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("Table: {}.{}", pol.schema_name, pol.table_name),
            kind: CompletionItemKind::Column,
            completion_text: None,
        };

        // autocomplete with the alias in a join clause if we find one
        if matches!(ctx.wrapping_clause_type, Some(WrappingClause::Join { .. })) {
            item.completion_text = find_matching_alias_for_table(ctx, pol.table_name.as_str())
                .and_then(|alias| {
                    get_completion_text_with_schema_or_alias(ctx, pol.name.as_str(), alias.as_str())
                });
        }

        builder.add_item(item);
    }
}
