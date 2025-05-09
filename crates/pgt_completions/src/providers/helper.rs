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

pub(crate) fn get_range_to_replace(ctx: &CompletionContext) -> TextRange {
    let start = ctx
        .node_under_cursor
        .as_ref()
        .map(|n| n.start_byte())
        .unwrap_or(0);

    let end = ctx
        .get_node_under_cursor_content()
        .unwrap_or("".into())
        .len()
        + start;

    TextRange::new(
        TextSize::new(start.try_into().unwrap()),
        end.try_into().unwrap(),
    )
}

pub(crate) fn get_completion_text_with_schema_or_alias(
    ctx: &CompletionContext,
    item_name: &str,
    schema_or_alias_name: &str,
) -> Option<CompletionText> {
    if schema_or_alias_name == "public" || ctx.schema_or_alias_name.is_some() {
        None
    } else {
        let range = get_range_to_replace(ctx);

        Some(CompletionText {
            text: format!("{}.{}", schema_or_alias_name, item_name),
            range,
        })
    }
}
