use pgls_schema_cache::{Column, SchemaCache};
use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::with_schema_or_alias;

pub fn complete_columns<'a>(
    ctx: &TreesitterContext<'a>,
    schema_cache: &'a SchemaCache,
    builder: &mut CompletionBuilder<'a>,
) {
    let available_columns = &schema_cache.columns;

    for col in available_columns {
        let relevance = CompletionRelevanceData::Column(col);

        let item = PossibleCompletionItem {
            label: col.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: format!("{}.{}", col.schema_name, col.table_name),
            kind: CompletionItemKind::Column,
            completion_text: Some(get_completion_text(ctx, col)),
            detail: col.type_name.as_ref().map(|t| t.to_string()),
        };

        builder.add_item(item);
    }
}

fn get_completion_text(ctx: &TreesitterContext, col: &Column) -> CompletionText {
    let alias = ctx.get_used_alias_for_table(col.table_name.as_str());

    let with_schema_or_alias = with_schema_or_alias(ctx, col.name.as_str(), alias.as_deref());

    let range = get_range_to_replace(ctx);

    CompletionText {
        is_snippet: false,
        range,
        text: with_schema_or_alias,
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use pgls_text_size::TextRange;
    use sqlx::{Executor, PgPool};

    use crate::{
        CompletionItem, CompletionItemKind, complete,
        test_helper::{
            CompletionAssertion, assert_complete_results, assert_no_complete_results,
            get_test_deps, get_test_params,
        },
    };

    use pgls_test_utils::QueryWithCursorPosition;

    struct TestCase {
        query: String,
        message: &'static str,
        label: &'static str,
        description: &'static str,
    }

    impl TestCase {
        fn get_input_query(&self) -> QueryWithCursorPosition {
            let strs: Vec<&str> = self.query.split_whitespace().collect();
            strs.join(" ").as_str().into()
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_columns(pool: PgPool) {
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

        pool.execute(setup).await.unwrap();

        let queries: Vec<TestCase> = vec![
            TestCase {
                message: "correctly prefers the columns of present tables",
                query: format!(
                    r#"select na{} from public.audio_books;"#,
                    QueryWithCursorPosition::cursor_marker()
                ),
                label: "narrator",
                description: "public.audio_books",
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
                    QueryWithCursorPosition::cursor_marker()
                ),
                label: "narrator_id",
                description: "private.audio_books",
            },
            TestCase {
                message: "works without a schema",
                query: format!(
                    r#"select na{} from users;"#,
                    QueryWithCursorPosition::cursor_marker()
                ),
                label: "name",
                description: "public.users",
            },
        ];

        for q in queries {
            let (tree, cache) = get_test_deps(None, q.get_input_query(), &pool).await;
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn shows_multiple_columns_if_no_relation_specified(pool: PgPool) {
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

        pool.execute(setup).await.unwrap();

        let case = TestCase {
            query: format!(r#"select n{};"#, QueryWithCursorPosition::cursor_marker()),
            description: "",
            label: "",
            message: "",
        };

        let (tree, cache) = get_test_deps(None, case.get_input_query(), &pool).await;
        let params = get_test_params(&tree, &cache, case.get_input_query());
        let mut items = complete(params);

        let _ = items.split_off(4);

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
            ("name", "public.users"),
            ("narrator", "public.audio_books"),
            ("narrator_id", "private.audio_books"),
            ("id", "public.audio_books"),
        ]
        .into_iter()
        .map(|(label, schema)| LabelAndDesc {
            label: label.into(),
            desc: schema.into(),
        })
        .collect::<Vec<LabelAndDesc>>();

        assert_eq!(labels, expected);
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_relevant_columns_without_letters(pool: PgPool) {
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
            query: format!(
                r#"select {} from users"#,
                QueryWithCursorPosition::cursor_marker()
            ),
            label: "",
            description: "",
        };

        let (tree, cache) = get_test_deps(Some(setup), test_case.get_input_query(), &pool).await;
        let params = get_test_params(&tree, &cache, test_case.get_input_query());
        let results = complete(params);

        let (first_four, _rest) = results.split_at(4);

        let has_column_in_first_four = |col: &'static str| {
            first_four
                .iter()
                .any(|compl_item| compl_item.label.as_str() == col)
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn ignores_cols_in_from_clause(pool: PgPool) {
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
            query: format!(
                r#"select * from private.{}"#,
                QueryWithCursorPosition::cursor_marker()
            ),
            label: "",
            description: "",
        };

        let (tree, cache) = get_test_deps(Some(setup), test_case.get_input_query(), &pool).await;
        let params = get_test_params(&tree, &cache, test_case.get_input_query());
        let results = complete(params);

        assert!(
            !results
                .into_iter()
                .any(|item| item.kind == CompletionItemKind::Column)
        );
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn prefers_columns_of_mentioned_tables(pool: PgPool) {
        let setup = r#"
        create schema private;

        create table private.users (
            id1 serial primary key,
            name1 text,
            address1 text,
            email1 text,
            user_settings jsonb
        );

        create table public.users (
            id2 serial primary key,
            name2 text,
            address2 text,
            email2 text,
            settings jsonb
        );
    "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!(
                r#"select {} from users"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("address2".into()),
                CompletionAssertion::Label("email2".into()),
                CompletionAssertion::Label("id2".into()),
                CompletionAssertion::Label("name2".into()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"select {} from private.users"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("address1".into()),
                CompletionAssertion::Label("email1".into()),
                CompletionAssertion::Label("id1".into()),
                CompletionAssertion::Label("name1".into()),
            ],
            None,
            &pool,
        )
        .await;

        // asserts fuzzy finding for "settings"
        assert_complete_results(
            format!(
                r#"select sett{} from private.users"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::Label("user_settings".into())],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn filters_out_by_aliases(pool: PgPool) {
        let setup = r#"
            create schema auth;

            create table auth.users (
                uid serial primary key,
                name text not null,
                email text unique not null
            );

            create table auth.posts (
                pid serial primary key,
                user_id int not null references auth.users(uid),
                title text not null,
                content text,
                created_at timestamp default now()
            );
        "#;

        pool.execute(setup).await.unwrap();

        // test in SELECT clause
        assert_complete_results(
            format!(
                "select u.id, p.{} from auth.users u join auth.posts p on u.id = p.user_id;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelNotExists("uid".to_string()),
                CompletionAssertion::LabelNotExists("name".to_string()),
                CompletionAssertion::LabelNotExists("email".to_string()),
                CompletionAssertion::Label("content".to_string()),
                CompletionAssertion::Label("created_at".to_string()),
                CompletionAssertion::Label("pid".to_string()),
                CompletionAssertion::Label("title".to_string()),
                CompletionAssertion::Label("user_id".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        // test in JOIN clause
        assert_complete_results(
            format!(
                "select u.id, p.content from auth.users u join auth.posts p on u.id = p.{};",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelNotExists("uid".to_string()),
                CompletionAssertion::LabelNotExists("name".to_string()),
                CompletionAssertion::LabelNotExists("email".to_string()),
                // primary keys are preferred
                CompletionAssertion::Label("pid".to_string()),
                CompletionAssertion::Label("content".to_string()),
                CompletionAssertion::Label("created_at".to_string()),
                CompletionAssertion::Label("title".to_string()),
                CompletionAssertion::Label("user_id".to_string()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn does_not_complete_cols_in_join_clauses(pool: PgPool) {
        let setup = r#"
            create schema auth;

            create table auth.users (
                uid serial primary key,
                name text not null,
                email text unique not null
            );

            create table auth.posts (
                pid serial primary key,
                user_id int not null references auth.users(uid),
                title text not null,
                content text,
                created_at timestamp default now()
            );
        "#;

        /*
         * We are not in the "ON" part of the JOIN clause, so we should not complete columns.
         */
        assert_complete_results(
            format!(
                "select u.id, p.content from auth.users u join auth.{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::KindNotExists(CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind("posts".to_string(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("users".to_string(), CompletionItemKind::Table),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_in_join_on_clause(pool: PgPool) {
        let setup = r#"
            create schema auth;

            create table auth.users (
                uid serial primary key,
                name text not null,
                email text unique not null
            );

            create table auth.posts (
                pid serial primary key,
                user_id int not null references auth.users(uid),
                title text not null,
                content text,
                created_at timestamp default now()
            );
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!(
                "select u.id, auth.posts.content from auth.users u join auth.posts on u.{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::KindNotExists(CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("uid".to_string(), CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind("email".to_string(), CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind("name".to_string(), CompletionItemKind::Column),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "select u.id, p.content from auth.users u join auth.posts p on p.user_id = u.{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::KindNotExists(CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("uid".to_string(), CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind("email".to_string(), CompletionItemKind::Column),
                CompletionAssertion::LabelAndKind("name".to_string(), CompletionItemKind::Column),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn prefers_not_mentioned_columns(pool: PgPool) {
        let setup = r#"
            create schema auth;

            create table public.one (
                id serial primary key,
                a text,
                b text,
                z text
            );

            create table public.two (
                id serial primary key,
                c text,
                d text,
                e text
            );
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!(
                "select {} from public.one o join public.two on o.id = t.id;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("a".to_string()),
                CompletionAssertion::Label("b".to_string()),
                CompletionAssertion::Label("c".to_string()),
                CompletionAssertion::Label("d".to_string()),
                CompletionAssertion::Label("e".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        // "a" is already mentioned, so it jumps down
        assert_complete_results(
            format!(
                "select a, {} from public.one o join public.two on o.id = t.id;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("b".to_string()),
                CompletionAssertion::Label("c".to_string()),
                CompletionAssertion::Label("d".to_string()),
                CompletionAssertion::Label("e".to_string()),
                CompletionAssertion::Label("id".to_string()),
                CompletionAssertion::Label("z".to_string()),
                CompletionAssertion::Label("a".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        // "id" of table one is mentioned, but table two isn't –
        // its priority stays up
        assert_complete_results(
            format!(
                "select o.id, a, b, c, d, e, {} from public.one o join public.two on o.id = t.id;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndDesc("id".to_string(), "public.two".to_string()),
                CompletionAssertion::Label("z".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        // "id" is ambiguous, so both "id" columns are lowered in priority
        assert_complete_results(
            format!(
                "select id, a, b, c, d, e, {} from public.one o join public.two on o.id = t.id;",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::Label("z".to_string())],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_columns_in_insert_clause(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text
            );

            create table others (
                id serial primary key,
                a text,
                b text
            );
        "#;

        pool.execute(setup).await.unwrap();

        // We should prefer the instrument columns, even though they
        // are lower in the alphabet

        assert_complete_results(
            format!(
                "insert into instruments ({})",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("id".to_string()),
                CompletionAssertion::Label("name".to_string()),
                CompletionAssertion::Label("z".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "insert into instruments (id, {})",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("name".to_string()),
                CompletionAssertion::Label("z".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "insert into instruments (id, {}, name)",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::Label("z".to_string())],
            None,
            &pool,
        )
        .await;

        // works with completed statement
        assert_complete_results(
            format!(
                "insert into instruments (name, {}) values ('my_bass');",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("id".to_string()),
                CompletionAssertion::Label("z".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        // no completions in the values list!
        assert_no_complete_results(
            format!(
                "insert into instruments (id, name) values ({})",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_columns_in_where_clause(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text, 
                created_at timestamp with time zone default now()
            );

            create table others (
                a text,
                b text,
                c text
            );
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!(
                "select name from instruments where {} ",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("created_at".into()),
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
                CompletionAssertion::Label("z".into()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "select name from instruments where z = 'something' and created_at > {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            // simply do not complete columns + schemas; functions etc. are ok
            vec![
                CompletionAssertion::KindNotExists(CompletionItemKind::Column),
                CompletionAssertion::KindNotExists(CompletionItemKind::Schema),
            ],
            None,
            &pool,
        )
        .await;

        // prefers not mentioned columns
        assert_complete_results(
            format!(
                "select name from instruments where id = 'something' and {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("created_at".into()),
                CompletionAssertion::Label("name".into()),
                CompletionAssertion::Label("z".into()),
            ],
            None,
            &pool,
        )
        .await;

        // // uses aliases
        assert_complete_results(
            format!(
                "select name from instruments i join others o on i.z = o.a where i.{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("created_at".into()),
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_columns_in_alter_table_and_drop_table(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text, 
                created_at timestamp with time zone default now()
            );

            create table others (
                a text,
                b text,
                c text
            );
        "#;

        pool.execute(setup).await.unwrap();

        let queries = vec![
            format!(
                "alter table instruments drop column {}",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table instruments drop column if exists {}",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table instruments alter column {} set default",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table instruments alter {} set default",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table public.instruments alter column {}",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table instruments alter {}",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table instruments rename {} to new_col",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                "alter table public.instruments rename column {} to new_col",
                QueryWithCursorPosition::cursor_marker()
            ),
        ];

        for query in queries {
            assert_complete_results(
                query.as_str(),
                vec![
                    CompletionAssertion::Label("created_at".into()),
                    CompletionAssertion::Label("id".into()),
                    CompletionAssertion::Label("name".into()),
                    CompletionAssertion::Label("z".into()),
                ],
                None,
                &pool,
            )
            .await;
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_columns_policy_using_clause(pool: PgPool) {
        let setup = r#"
            create table instruments (
                id bigint primary key generated always as identity,
                name text not null,
                z text, 
                created_at timestamp with time zone default now()
            );
        "#;

        pool.execute(setup).await.unwrap();

        let col_queries = vec![
            format!(
                r#"create policy "my_pol" on public.instruments for select using ({})"#,
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                r#"create policy "my_pol" on public.instruments for insert with check ({})"#,
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                r#"create policy "my_pol" on public.instruments for update using (id = 1 and {})"#,
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                r#"create policy "my_pol" on public.instruments for insert with check (id = 1 and {})"#,
                QueryWithCursorPosition::cursor_marker()
            ),
        ];

        for query in col_queries {
            assert_complete_results(
                query.as_str(),
                vec![
                    CompletionAssertion::Label("created_at".into()),
                    CompletionAssertion::Label("id".into()),
                    CompletionAssertion::Label("name".into()),
                    CompletionAssertion::Label("z".into()),
                ],
                None,
                &pool,
            )
            .await;
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_quoted_columns(pool: PgPool) {
        let setup = r#"
            create schema if not exists private;
            
            create table private.users (
                id serial primary key,
                email text unique not null,
                name text not null,
                "quoted_column" text
            );
        "#;

        pool.execute(setup).await.unwrap();

        // test completion inside quoted column name
        assert_complete_results(
            format!(
                r#"select "em{}" from "private"."users""#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::LabelAndDesc(
                "email".to_string(),
                "private.users".to_string(),
            )],
            None,
            &pool,
        )
        .await;

        // test completion for already quoted column
        assert_complete_results(
            format!(
                r#"select "quoted_col{}" from "private"."users""#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::LabelAndDesc(
                "quoted_column".to_string(),
                "private.users".to_string(),
            )],
            None,
            &pool,
        )
        .await;

        // test completion with empty quotes
        assert_complete_results(
            format!(
                r#"select "{}" from "private"."users""#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("email".to_string()),
                CompletionAssertion::Label("id".to_string()),
                CompletionAssertion::Label("name".to_string()),
                CompletionAssertion::Label("quoted_column".to_string()),
            ],
            None,
            &pool,
        )
        .await;

        // test completion with partially opened quote
        assert_complete_results(
            format!(
                r#"select "{} from "private"."users""#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("email".to_string()),
                CompletionAssertion::Label("id".to_string()),
                CompletionAssertion::Label("name".to_string()),
                CompletionAssertion::Label("quoted_column".to_string()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_quoted_columns_with_aliases(pool: PgPool) {
        let setup = r#"
            create schema if not exists private;
            
            create table private.users (
                id serial primary key,
                email text unique not null,
                name text not null,
                "quoted_column" text
            );

            create table public.names (
                uid serial references private.users(id),
                name text
            );
        "#;

        pool.execute(setup).await.unwrap();

        {
            // should suggest pr"."email and insert into existing quotes
            let query = format!(
                r#"select "e{}" from private.users "pr""#,
                QueryWithCursorPosition::cursor_marker()
            );

            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    r#"pr"."email"#.into(),
                    // replaces the full `"e"`
                    TextRange::new(8.into(), 9.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest pr"."email and insert into existing quotes
            let query = format!(
                r#"select "{}" from private.users "pr""#,
                QueryWithCursorPosition::cursor_marker()
            );

            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    r#"pr"."email"#.into(),
                    TextRange::new(8.into(), 8.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest email and insert into quotes
            let query = format!(
                r#"select pr."{}" from private.users "pr""#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    r#"email"#.into(),
                    TextRange::new(11.into(), 11.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest email
            let query = format!(
                r#"select "pr".{} from private.users "pr""#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    "email".into(),
                    TextRange::new(12.into(), 12.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest `email`
            let query = format!(
                r#"select pr.{} from private.users "pr""#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    "email".into(),
                    TextRange::new(10.into(), 10.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            let query = format!(
                r#"select {} from private.users "pr" join public.names n on pr.id = n.uid;"#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![
                    CompletionAssertion::CompletionTextAndRange(
                        "n.name".into(),
                        TextRange::new(7.into(), 7.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        "n.uid".into(),
                        TextRange::new(7.into(), 7.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        r#""pr".email"#.into(),
                        TextRange::new(7.into(), 7.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        r#""pr".id"#.into(),
                        TextRange::new(7.into(), 7.into()),
                    ),
                ],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest "pr"."email"
            let query = format!(
                r#"select "{}" from private.users "pr" join public.names "n" on pr.id = n.uid;"#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![
                    CompletionAssertion::CompletionTextAndRange(
                        r#"n"."name"#.into(),
                        TextRange::new(8.into(), 8.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        r#"n"."uid"#.into(),
                        TextRange::new(8.into(), 8.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        r#"pr"."email"#.into(),
                        TextRange::new(8.into(), 8.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        r#"pr"."id"#.into(),
                        TextRange::new(8.into(), 8.into()),
                    ),
                ],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest pr"."email"
            let query = format!(
                r#"select "{} from private.users "pr";"#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![
                    CompletionAssertion::CompletionTextAndRange(
                        r#"pr"."email""#.into(),
                        TextRange::new(8.into(), 8.into()),
                    ),
                    CompletionAssertion::CompletionTextAndRange(
                        r#"pr"."id""#.into(),
                        TextRange::new(8.into(), 8.into()),
                    ),
                ],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest email"
            let query = format!(
                r#"select pr."{} from private.users "pr";"#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    r#"email""#.into(),
                    TextRange::new(11.into(), 11.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest email"
            let query = format!(
                r#"select "pr"."{} from private.users "pr";"#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    r#"email""#.into(),
                    TextRange::new(13.into(), 13.into()),
                )],
                None,
                &pool,
            )
            .await;
        }

        {
            // should suggest "n".name
            let query = format!(
                r#"select {} from names "n";"#,
                QueryWithCursorPosition::cursor_marker()
            );
            assert_complete_results(
                query.as_str(),
                vec![CompletionAssertion::CompletionTextAndRange(
                    r#""n".name"#.into(),
                    TextRange::new(7.into(), 7.into()),
                )],
                None,
                &pool,
            )
            .await;
        }
    }
}
