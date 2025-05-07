use pgt_text_size::{TextRange, TextSize};

use crate::{CompletionText, context::CompletionContext};

pub(crate) fn find_matching_alias_for_table(
    ctx: &CompletionContext,
    table_name: &str,
) -> Option<String> {
    for (alias, table) in ctx.mentioned_table_aliases.iter() {
        if table == table_name {
            return Some(alias.to_string());
        }
    }
    None
}

pub(crate) fn get_completion_text_with_schema_or_alias(
    ctx: &CompletionContext,
    item_name: &str,
    item_schema_name: &str,
) -> Option<CompletionText> {
    if item_schema_name == "public" || ctx.schema_or_alias_name.is_some() {
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
