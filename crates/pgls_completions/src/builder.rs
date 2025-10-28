use crate::{
    CompletionItemKind, CompletionText,
    item::CompletionItem,
    relevance::{filtering::CompletionFilter, scoring::CompletionScore},
};

use pgls_treesitter::TreesitterContext;

#[derive(Debug)]
pub(crate) struct PossibleCompletionItem<'a> {
    pub label: String,
    pub description: String,
    pub kind: CompletionItemKind,
    pub score: CompletionScore<'a>,
    pub filter: CompletionFilter<'a>,
    pub completion_text: Option<CompletionText>,
    pub detail: Option<String>,
}

pub(crate) struct CompletionBuilder<'a> {
    items: Vec<PossibleCompletionItem<'a>>,
    ctx: &'a TreesitterContext<'a>,
}

impl<'a> CompletionBuilder<'a> {
    pub fn new(ctx: &'a TreesitterContext) -> Self {
        CompletionBuilder { items: vec![], ctx }
    }

    pub fn add_item(&mut self, item: PossibleCompletionItem<'a>) {
        self.items.push(item);
    }

    pub fn finish(self) -> Vec<CompletionItem> {
        let mut items: Vec<PossibleCompletionItem> = self
            .items
            .into_iter()
            .filter(|i| i.filter.is_relevant(self.ctx).is_some())
            .collect();

        for item in items.iter_mut() {
            item.score.calc_score(self.ctx);
        }

        items.sort_by(|a, b| {
            b.score
                .get_score()
                .cmp(&a.score.get_score())
                .then_with(|| a.label.cmp(&b.label))
        });

        items.dedup_by(|a, b| a.label == b.label);
        items.truncate(crate::LIMIT);

        let should_preselect_first_item = should_preselect_first_item(&items);

        /*
         * LSP Clients themselves sort the completion items.
         * They'll use the `sort_text` property if present (or fallback to the `label`).
         * Since our items are already sorted, we're 'hijacking' the sort_text.
         * We're simply adding the index of the item, padded by zeroes to the max length.
         */
        let max_padding = items.len().to_string().len();

        items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| {
                let preselected = idx == 0 && should_preselect_first_item;

                CompletionItem {
                    description: item.description,
                    kind: item.kind,
                    label: item.label,
                    preselected,
                    detail: item.detail,

                    // wonderous Rust syntax ftw
                    sort_text: format!("{idx:0>max_padding$}"),
                    completion_text: item.completion_text,
                }
            })
            .collect()
    }
}

fn should_preselect_first_item(items: &Vec<PossibleCompletionItem>) -> bool {
    let mut items_iter = items.iter();
    let first = items_iter.next();
    let second = items_iter.next();

    first.is_some_and(|f| match second {
        Some(s) => (f.score.get_score() - s.score.get_score()) > 15,
        None => true,
    }) && items.len() >= 10
}
