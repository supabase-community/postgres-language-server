use pgt_text_size::{TextRange, TextSize};
use pgt_treesitter::TreesitterContext;

use crate::{is_sanitized_token_with_quote, remove_sanitized_token};

pub(crate) fn find_matching_alias_for_table(
    ctx: &TreesitterContext,
    table_name: &str,
) -> Option<String> {
    for (alias, table) in ctx.mentioned_table_aliases.iter() {
        if table == table_name {
            return Some(alias.to_string());
        }
    }
    None
}

pub(crate) fn get_range_to_replace(ctx: &TreesitterContext) -> TextRange {
    match ctx.node_under_cursor.as_ref() {
        Some(node) => {
            let content = ctx.get_node_under_cursor_content().unwrap_or("".into());
            let content = content.as_str();

            let length = remove_sanitized_token(content).len();

            let start = node.start_byte();
            let mut end = start + length;

            if is_sanitized_token_with_quote(content) {
                end += 1;
            }

            TextRange::new(start.try_into().unwrap(), end.try_into().unwrap())
        }
        None => TextRange::empty(TextSize::new(0)),
    }
}

pub(crate) fn with_schema_or_alias(
    ctx: &TreesitterContext,
    item_name: &str,
    schema_or_alias_name: Option<&str>,
) -> String {
    let is_already_prefixed_with_schema_name = ctx.schema_or_alias_name.is_some();

    if schema_or_alias_name.is_none_or(|s| s == "public") || is_already_prefixed_with_schema_name {
        item_name.to_string()
    } else {
        format!("{}.{}", schema_or_alias_name.unwrap(), item_name).to_string()
    }
}

pub(crate) fn with_closed_quote(ctx: &TreesitterContext, item_name: &str) -> String {
    let mut with_closed = String::from(item_name);

    if let Some(content) = ctx.get_node_under_cursor_content() {
        if is_sanitized_token_with_quote(content.as_str()) {
            with_closed.push('"');
        }
    }

    with_closed
}
