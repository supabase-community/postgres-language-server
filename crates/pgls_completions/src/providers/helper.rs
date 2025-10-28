use pgls_text_size::{TextRange, TextSize};
use pgls_treesitter::TreesitterContext;

use crate::{is_sanitized_token_with_quote, remove_sanitized_token};

pub(crate) fn node_text_surrounded_by_quotes(ctx: &TreesitterContext) -> bool {
    ctx.get_node_under_cursor_content()
        .is_some_and(|c| c.starts_with('"') && c.ends_with('"') && c.len() > 1)
}

pub(crate) fn get_range_to_replace(ctx: &TreesitterContext) -> TextRange {
    match ctx.node_under_cursor.as_ref() {
        Some(node) => {
            let content = ctx.get_node_under_cursor_content().unwrap_or("".into());
            let content = content.as_str();

            let sanitized = remove_sanitized_token(content);
            let length = sanitized.len();

            let mut start = node.start_byte();
            let mut end = start + length;

            if sanitized.starts_with('"') && sanitized.ends_with('"') {
                start += 1;

                if sanitized.len() > 1 {
                    end -= 1;
                }
            }

            TextRange::new(start.try_into().unwrap(), end.try_into().unwrap())
        }
        None => TextRange::empty(TextSize::new(0)),
    }
}

pub(crate) fn only_leading_quote(ctx: &TreesitterContext) -> bool {
    let node_under_cursor_txt = ctx.get_node_under_cursor_content().unwrap_or("".into());
    let node_under_cursor_txt = node_under_cursor_txt.as_str();
    is_sanitized_token_with_quote(node_under_cursor_txt)
}

pub(crate) fn with_schema_or_alias(
    ctx: &TreesitterContext,
    item_name: &str,
    schema_or_alias_name: Option<&str>,
) -> String {
    let is_already_prefixed_with_schema_name = ctx.schema_or_alias_name.is_some();

    let with_quotes = node_text_surrounded_by_quotes(ctx);
    let single_leading_quote = only_leading_quote(ctx);

    if schema_or_alias_name.is_none_or(|s| s == "public") || is_already_prefixed_with_schema_name {
        if single_leading_quote {
            format!(r#"{item_name}""#)
        } else {
            item_name.to_string()
        }
    } else {
        let schema_or_als = schema_or_alias_name.unwrap();

        if single_leading_quote {
            format!(r#"{}"."{}""#, schema_or_als.replace('"', ""), item_name)
        } else if with_quotes {
            format!(r#"{}"."{}"#, schema_or_als.replace('"', ""), item_name)
        } else {
            format!("{schema_or_als}.{item_name}")
        }
    }
}
