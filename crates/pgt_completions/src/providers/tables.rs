use crate::{
    builder::CompletionBuilder,
    context::CompletionContext,
    item::{CompletionItem, CompletionItemKind},
    relevance::CompletionRelevanceData,
};

pub fn complete_tables(ctx: &CompletionContext, builder: &mut CompletionBuilder) {
    let available_tables = &ctx.schema_cache.tables;

    for table in available_tables {
        let item = CompletionItem {
            label: table.name.clone(),
            score: CompletionRelevanceData::Table(table).get_score(ctx),
            description: format!("Schema: {}", table.schema),
            preselected: false,
            kind: CompletionItemKind::Table,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        CompletionItem, CompletionItemKind, complete,
        test_helper::{CURSOR_POS, get_test_deps, get_test_params},
    };

    #[tokio::test]
    async fn autocompletes_simple_table() {
        let setup = r#"
            create table users (
                id serial primary key,
                name text,
                password text
            );
        "#;

        let query = format!("select * from u{}", CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let items = complete(params);

        assert!(!items.is_empty());

        let best_match = &items[0];

        assert_eq!(
            best_match.label, "users",
            "Does not return the expected table to autocomplete: {}",
            best_match.label
        )
    }

    #[tokio::test]
    async fn autocompletes_table_alphanumerically() {
        let setup = r#"
            create table addresses (
                id serial primary key
            );

            create table users (
                id serial primary key
            );

            create table emails (
                id serial primary key
            );
        "#;

        let test_cases = vec![
            (format!("select * from u{}", CURSOR_POS), "users"),
            (format!("select * from e{}", CURSOR_POS), "emails"),
            (format!("select * from a{}", CURSOR_POS), "addresses"),
        ];

        for (query, expected_label) in test_cases {
            let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
            let params = get_test_params(&tree, &cache, query.as_str().into());
            let items = complete(params);

            assert!(!items.is_empty());

            let best_match = &items[0];

            assert_eq!(
                best_match.label, expected_label,
                "Does not return the expected table to autocomplete: {}",
                best_match.label
            )
        }
    }

    #[tokio::test]
    async fn autocompletes_table_with_schema() {
        let setup = r#"
            create schema customer_support;
            create schema private;

            create table private.user_z (
                id serial primary key,
                name text,
                password text
            );

            create table customer_support.user_y (
                id serial primary key,
                request text,
                send_at timestamp with time zone
            );
        "#;

        let test_cases = vec![
            (format!("select * from u{}", CURSOR_POS), "user_y"), // user_y is preferred alphanumerically
            (format!("select * from private.u{}", CURSOR_POS), "user_z"),
            (
                format!("select * from customer_support.u{}", CURSOR_POS),
                "user_y",
            ),
        ];

        for (query, expected_label) in test_cases {
            let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
            let params = get_test_params(&tree, &cache, query.as_str().into());
            let items = complete(params);

            assert!(!items.is_empty());

            let best_match = &items[0];

            assert_eq!(
                best_match.label, expected_label,
                "Does not return the expected table to autocomplete: {}",
                best_match.label
            )
        }
    }

    #[tokio::test]
    async fn prefers_table_in_from_clause() {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );

          create or replace function cool()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;
        "#;

        let query = format!(r#"select * from coo{}"#, CURSOR_POS);

        let (tree, cache) = get_test_deps(setup, query.as_str().into()).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let items = complete(params);

        let CompletionItem { label, kind, .. } = items
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "coos");
        assert_eq!(kind, CompletionItemKind::Table);
    }
}
