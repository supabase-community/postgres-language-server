use std::ops::Range;

use pgls_text_size::TextRange;
use pgls_treesitter::TreesitterContext;

use crate::{is_sanitized_token_with_quote, remove_sanitized_token};

pub(crate) fn node_text_surrounded_by_quotes(ctx: &TreesitterContext) -> bool {
    ctx.get_node_under_cursor_content()
        .is_some_and(|c| c.starts_with('"') && c.ends_with('"') && c.len() > 1)
}

pub(crate) fn get_range_to_replace(ctx: &TreesitterContext) -> TextRange {
    let node = &ctx.node_under_cursor;
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

/// The bytes of the word at the cursor, like `ver` in `explain (analyze, ver|)`, or the
/// sanitized token if the cursor is on one. A keyword completion replaces this word.
///
/// It isn't always the node under the cursor: where the grammar expects no identifier, as in
/// option lists, tree-sitter puts the word into an `ERROR` together with the tokens before it,
/// like `, ver`, or skips it, so that the node under the cursor is the `)` after it.
pub(crate) fn word_at_cursor(ctx: &TreesitterContext) -> Range<usize> {
    let bytes = ctx.text.as_bytes();
    let is_word = |b: &u8| b.is_ascii_alphanumeric() || *b == b'_';

    if !bytes.get(ctx.position).is_some_and(is_word) {
        return ctx.node_under_cursor.byte_range();
    }

    let start = bytes[..ctx.position]
        .iter()
        .rposition(|b| !is_word(b))
        .map_or(0, |i| i + 1);
    let end = bytes[ctx.position..]
        .iter()
        .position(|b| !is_word(b))
        .map_or(bytes.len(), |i| ctx.position + i);

    start..end
}

/// The range a keyword completion replaces: the [`word_at_cursor`] without the sanitized token.
pub(crate) fn get_keyword_range_to_replace(ctx: &TreesitterContext) -> TextRange {
    let word = word_at_cursor(ctx);
    let typed = remove_sanitized_token(&ctx.text[word.clone()]);

    TextRange::at(
        word.start.try_into().unwrap(),
        typed.len().try_into().unwrap(),
    )
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
    let is_already_prefixed_with_schema_name = ctx.has_any_qualifier();

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
