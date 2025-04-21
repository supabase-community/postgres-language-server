use crate::{
    CompletionItemKind,
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

pub fn complete_columns<'a>(ctx: &CompletionContext<'a>, builder: &mut CompletionBuilder<'a>) {
    let available_columns = &ctx.schema_cache.columns;

    for col in available_columns {
        let relevance = CompletionRelevanceData::Column(col);

        let item = PossibleCompletionItem {
            label: col.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("Table: {}.{}", col.schema_name, col.table_name),
            kind: CompletionItemKind::Column,
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CompletionItem, CompletionItemKind, complete,
        test_helper::{
            CURSOR_POS, InputQuery, get_test_deps, get_test_params, test_against_connection_string,
        },
    };

    struct TestCase {
        query: String,
        message: &'static str,
        label: &'static str,
        description: &'static str,
    }

    impl TestCase {
        fn get_input_query(&self) -> InputQuery {
            let strs: Vec<&str> = self.query.split_whitespace().collect();
            strs.join(" ").as_str().into()
        }
    }

    #[tokio::test]
    async fn completes_columns() {
        let setup = r#"
            create schema private;

            create table public.users (
                id serial primary key,
                name text
            );

            create table public.audio_books (
                id serial primary key,
                narrator text
            );

            create table private.audio_books (
                id serial primary key,
                narrator_id text
            );
        "#;

        let queries: Vec<TestCase> = vec![
            TestCase {
                message: "correctly prefers the columns of present tables",
                query: format!(r#"select na{} from public.audio_books;"#, CURSOR_POS),
                label: "narrator",
                description: "Table: public.audio_books",
            },
            TestCase {
                message: "correctly handles nested queries",
                query: format!(
                    r#"
                select
                    *
                from (
                    select id, na{}
                    from private.audio_books
                ) as subquery
                join public.users u
                on u.id = subquery.id;
                "#,
                    CURSOR_POS
                ),
                label: "narrator_id",
                description: "Table: private.audio_books",
            },
            TestCase {
                message: "works without a schema",
                query: format!(r#"select na{} from users;"#, CURSOR_POS),
                label: "name",
                description: "Table: public.users",
            },
        ];

        for q in queries {
            let (tree, cache) = get_test_deps(setup, q.get_input_query()).await;
            let params = get_test_params(&tree, &cache, q.get_input_query());
            let results = complete(params);

            let CompletionItem {
                label, description, ..
            } = results
                .into_iter()
                .next()
                .expect("Should return at least one completion item");

            assert_eq!(label, q.label, "{}", q.message);
            assert_eq!(description, q.description, "{}", q.message);
        }
    }

    #[tokio::test]
    async fn shows_multiple_columns_if_no_relation_specified() {
        let setup = r#"
            create schema private;

            create table public.users (
                id serial primary key,
                name text
            );

            create table public.audio_books (
                id serial primary key,
                narrator text
            );

            create table private.audio_books (
                id serial primary key,
                narrator_id text
            );
        "#;

        let case = TestCase {
            query: format!(r#"select n{};"#, CURSOR_POS),
            description: "",
            label: "",
            message: "",
        };

        let (tree, cache) = get_test_deps(setup, case.get_input_query()).await;
        let params = get_test_params(&tree, &cache, case.get_input_query());
        let mut items = complete(params);

        let _ = items.split_off(6);

        #[derive(Eq, PartialEq, Debug)]
        struct LabelAndDesc {
            label: String,
            desc: String,
        }

        let labels: Vec<LabelAndDesc> = items
            .into_iter()
            .map(|c| LabelAndDesc {
                label: c.label,
                desc: c.description,
            })
            .collect();

        let expected = vec![
            ("name", "Table: public.users"),
            ("narrator", "Table: public.audio_books"),
            ("narrator_id", "Table: private.audio_books"),
            ("name", "Schema: pg_catalog"),
            ("nameconcatoid", "Schema: pg_catalog"),
            ("nameeq", "Schema: pg_catalog"),
        ]
        .into_iter()
        .map(|(label, schema)| LabelAndDesc {
            label: label.into(),
            desc: schema.into(),
        })
        .collect::<Vec<LabelAndDesc>>();

        assert_eq!(labels, expected);
    }

    #[tokio::test]
    async fn suggests_relevant_columns_without_letters() {
        let setup = r#"
            create table users (
                id serial primary key,
                name text,
                address text,
                email text
            );
        "#;

        let test_case = TestCase {
            message: "suggests user created tables first",
            query: format!(r#"select {} from users"#, CURSOR_POS),
            label: "",
            description: "",
        };

        let (tree, cache) = get_test_deps(setup, test_case.get_input_query()).await;
        let params = get_test_params(&tree, &cache, test_case.get_input_query());
        let results = complete(params);

        let (first_four, _rest) = results.split_at(4);

        let has_column_in_first_four = |col: &'static str| {
            first_four
                .iter()
                .find(|compl_item| compl_item.label.as_str() == col)
                .is_some()
        };

        assert!(
            has_column_in_first_four("id"),
            "`id` not present in first four completion items."
        );
        assert!(
            has_column_in_first_four("name"),
            "`name` not present in first four completion items."
        );
        assert!(
            has_column_in_first_four("address"),
            "`address` not present in first four completion items."
        );
        assert!(
            has_column_in_first_four("email"),
            "`email` not present in first four completion items."
        );
    }

    #[tokio::test]
    async fn ignores_cols_in_from_clause() {
        let setup = r#"
        create schema private;

        create table private.users (
            id serial primary key,
            name text,
            address text,
            email text
        );
    "#;

        let test_case = TestCase {
            message: "suggests user created tables first",
            query: format!(r#"select * from private.{}"#, CURSOR_POS),
            label: "",
            description: "",
        };

        let (tree, cache) = get_test_deps(setup, test_case.get_input_query()).await;
        let params = get_test_params(&tree, &cache, test_case.get_input_query());
        let results = complete(params);

        assert!(
            !results
                .into_iter()
                .any(|item| item.kind == CompletionItemKind::Column)
        );
    }

    #[tokio::test]
    async fn prefers_columns_of_mentioned_tables() {
        let setup = r#"
        create schema private;

        create table private.users (
            id1 serial primary key,
            name1 text,
            address1 text,
            email1 text
        );

        create table public.users (
            id2 serial primary key,
            name2 text,
            address2 text,
            email2 text
        );
    "#;

        {
            let test_case = TestCase {
                message: "",
                query: format!(r#"select {} from users"#, CURSOR_POS),
                label: "suggests from table",
                description: "",
            };

            let (tree, cache) = get_test_deps(setup, test_case.get_input_query()).await;
            let params = get_test_params(&tree, &cache, test_case.get_input_query());
            let results = complete(params);

            assert_eq!(
                results
                    .into_iter()
                    .take(4)
                    .map(|item| item.label)
                    .collect::<Vec<String>>(),
                vec!["address2", "email2", "id2", "name2"]
            );
        }

        {
            let test_case = TestCase {
                message: "",
                query: format!(r#"select {} from private.users"#, CURSOR_POS),
                label: "suggests from table",
                description: "",
            };

            let (tree, cache) = get_test_deps(setup, test_case.get_input_query()).await;
            let params = get_test_params(&tree, &cache, test_case.get_input_query());
            let results = complete(params);

            assert_eq!(
                results
                    .into_iter()
                    .take(4)
                    .map(|item| item.label)
                    .collect::<Vec<String>>(),
                vec!["address1", "email1", "id1", "name1"]
            );
        }
    }
}
