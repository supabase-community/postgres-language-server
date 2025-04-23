use pgt_text_size::{TextRange, TextSize};

use crate::{CompletionText, context::CompletionContext};

pub(crate) fn get_completion_text_with_schema(
    ctx: &CompletionContext,
    item_name: &str,
    item_schema_name: &str,
) -> Option<CompletionText> {
    if item_schema_name == "public" {
        None
    } else if ctx.schema_name.is_some() {
        None
    } else {
        let node = ctx.node_under_cursor.unwrap();

        let range = TextRange::new(
            TextSize::try_from(node.start_byte()).unwrap(),
            TextSize::try_from(node.end_byte()).unwrap(),
        );

        Some(CompletionText {
            text: format!("{}.{}", item_schema_name, item_name),
            range,
        })
    }
}
