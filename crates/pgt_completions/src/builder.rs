use crate::item::CompletionItem;

pub(crate) struct CompletionBuilder {
    items: Vec<CompletionItem>,
}

impl CompletionBuilder {
    pub fn new() -> Self {
        CompletionBuilder { items: vec![] }
    }

    pub fn add_item(&mut self, item: CompletionItem) {
        self.items.push(item);
    }

    pub fn finish(mut self) -> Vec<CompletionItem> {
        self.items
            .sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.label.cmp(&b.label)));

        self.items.dedup_by(|a, b| a.label == b.label);
        self.items.truncate(crate::LIMIT);

        let should_preselect_first_item = self.should_preselect_first_item();

        self.items
            .into_iter()
            .enumerate()
            .map(|(idx, mut item)| {
                if idx == 0 {
                    item.preselected = should_preselect_first_item;
                }
                item
            })
            .collect()
    }

    fn should_preselect_first_item(&mut self) -> bool {
        let mut items_iter = self.items.iter();
        let first = items_iter.next();
        let second = items_iter.next();

        first.is_some_and(|f| match second {
            Some(s) => (f.score - s.score) > 10,
            None => true,
        })
    }
}
