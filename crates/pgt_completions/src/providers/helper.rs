use pgt_text_size::{TextRange, TextSize};
use pgt_treesitter::CompletionContext;

use crate::{CompletionText, remove_sanitized_token};

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

pub(crate) fn get_range_to_replace(ctx: &CompletionContext) -> TextRange {
    match ctx.node_under_cursor.as_ref() {
        Some(node) => {
            let content = ctx.get_node_under_cursor_content().unwrap_or("".into());
            let length = remove_sanitized_token(content.as_str()).len();

            let start = node.start_byte();
            let end = start + length;

            TextRange::new(start.try_into().unwrap(), end.try_into().unwrap())
        }
        None => TextRange::empty(TextSize::new(0)),
    }
}

pub(crate) fn get_completion_text_with_schema_or_alias(
    ctx: &CompletionContext,
    item_name: &str,
    schema_or_alias_name: &str,
) -> Option<CompletionText> {
    let is_already_prefixed_with_schema_name = ctx.schema_or_alias_name.is_some();

    if schema_or_alias_name == "public" || is_already_prefixed_with_schema_name {
        None
    } else {
        let range = get_range_to_replace(ctx);

        Some(CompletionText {
            text: format!("{}.{}", schema_or_alias_name, item_name),
            range,
            is_snippet: false,
        })
    }
}
