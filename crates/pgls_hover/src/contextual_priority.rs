use pgls_treesitter::context::TreesitterContext;

pub(crate) trait ContextualPriority {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32;
}

/// Will first sort the items by a score and then filter out items with a score gap algorithm.  
///
/// `[200, 180, 150, 140]` => all items are returned
///
/// `[200, 180, 15, 10]` => first two items are returned
///
/// `[200, 30, 20, 10]` => only first item is returned
pub(crate) fn prioritize_by_context<T: ContextualPriority + std::fmt::Debug>(
    items: Vec<T>,
    ctx: &TreesitterContext,
) -> Vec<T> {
    let mut scored: Vec<_> = items
        .into_iter()
        .map(|item| {
            let score = item.relevance_score(ctx);
            (item, score)
        })
        .collect();

    scored.sort_by(|(_, score_a), (_, score_b)| score_b.partial_cmp(score_a).unwrap());

    if scored.is_empty() {
        return vec![];
    }

    // always include the top result
    let top_result = scored.remove(0);
    let mut results = vec![top_result.0];
    let mut prev_score = top_result.1;

    // include additional results until we hit a significant score gap of 30%
    for (item, score) in scored.into_iter() {
        let gap = prev_score - score;
        if gap > prev_score * 0.3 {
            break;
        }
        results.push(item);
        prev_score = score;
    }

    results
}
