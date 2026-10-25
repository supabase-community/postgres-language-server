use pgls_catalog::{NamePosition, quote_identifier};
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

pub(crate) fn only_leading_quote(ctx: &TreesitterContext) -> bool {
    let node_under_cursor_txt = ctx.get_node_under_cursor_content().unwrap_or("".into());
    let node_under_cursor_txt = node_under_cursor_txt.as_str();
    is_sanitized_token_with_quote(node_under_cursor_txt)
}

/// The text to insert for `item_name`, prefixed with the schema or alias name unless it is
/// `public` or the user already wrote a qualifier. Names are quoted where Postgres would read
/// another name unquoted. `position` is where an unqualified `item_name` stands.
pub(crate) fn with_schema_or_alias(
    ctx: &TreesitterContext,
    item_name: &str,
    schema_or_alias_name: Option<&str>,
    position: NamePosition,
) -> String {
    let is_already_prefixed_with_schema_name = ctx.has_any_qualifier();

    let with_quotes = node_text_surrounded_by_quotes(ctx);
    let single_leading_quote = only_leading_quote(ctx);

    // Inside quotes the user already typed, a name only needs its `"` escaped.
    let escaped = |name: &str| name.replace('"', r#""""#);

    match schema_or_alias_name {
        Some(prefix) if prefix != "public" && !is_already_prefixed_with_schema_name => {
            if single_leading_quote {
                format!(r#"{}"."{}""#, escaped(prefix), escaped(item_name))
            } else if with_quotes {
                format!(r#"{}"."{}"#, escaped(prefix), escaped(item_name))
            } else {
                format!(
                    "{}.{}",
                    quote_identifier(prefix, NamePosition::ColId),
                    quote_identifier(item_name, NamePosition::ColLabel)
                )
            }
        }
        _ => {
            if single_leading_quote {
                format!(r#"{}""#, escaped(item_name))
            } else if with_quotes {
                escaped(item_name)
            } else if is_already_prefixed_with_schema_name {
                quote_identifier(item_name, NamePosition::ColLabel).into_owned()
            } else {
                quote_identifier(item_name, position).into_owned()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pgls_test_utils::QueryWithCursorPosition;
    use sqlx::{Executor, PgPool};

    use crate::{
        complete,
        test_helper::{get_test_deps, get_test_params},
    };

    /// The text inserted for the completion item `label` at `{c}` in `query`.
    async fn inserted(pool: &PgPool, query: &str, label: &str) -> Option<String> {
        let query = query.replace("{c}", &QueryWithCursorPosition::cursor_marker().to_string());
        let (tree, cache) = get_test_deps(None, query.as_str().into(), pool).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        complete(params)
            .into_iter()
            .find(|item| item.label == label)
            .map(|item| item.completion_text.map_or(item.label, |text| text.text))
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn quotes_names_that_need_it(pool: PgPool) {
        pool.execute(
            r#"
            create schema "My Schema";
            create table "MyTable" ("Id" int, "select" int, plain int);
            create table "My Schema"."Other" (x int);
            create function "MyFunc"() returns int language sql as 'select 1';
            create function "My Schema".helper() returns int language sql as 'select 1';
            "#,
        )
        .await
        .unwrap();

        for (query, label, expected) in [
            // Tables and schemas.
            ("select * from {c}", "MyTable", r#""MyTable""#),
            ("select * from {c}", "My Schema", r#""My Schema""#),
            ("select * from {c}", "Other", r#""My Schema"."Other""#),
            (r#"select * from "My Schema".{c}"#, "Other", r#""Other""#),
            // After an opening quote, only the closing one is missing.
            (r#"select * from "{c}"#, "MyTable", r#"MyTable""#),
            (r#"select * from "{c}"#, "Other", r#"My Schema"."Other""#),
            // Columns. A keyword needs quotes on its own, but not after a dot.
            (r#"select {c} from "MyTable""#, "Id", r#""Id""#),
            (r#"select {c} from "MyTable""#, "select", r#""select""#),
            (r#"select {c} from "MyTable""#, "plain", "plain"),
            (r#"select t.{c} from "MyTable" t"#, "select", "select"),
            (r#"select t.{c} from "MyTable" t"#, "Id", r#""Id""#),
            // Functions.
            ("select {c}", "MyFunc", r#""MyFunc"()"#),
            ("select {c}", "helper", r#""My Schema".helper()"#),
        ] {
            assert_eq!(
                inserted(&pool, query, label).await.as_deref(),
                Some(expected),
                "{query} → {label}"
            );
        }
    }
}
