use pgls_schema_cache::{SchemaCache, Table};
use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    item::CompletionItemKind,
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::with_schema_or_alias;

pub fn complete_tables<'a>(
    ctx: &'a TreesitterContext,
    schema_cache: &'a SchemaCache,
    builder: &mut CompletionBuilder<'a>,
) {
    let available_tables = &schema_cache.tables;

    for table in available_tables {
        let relevance = CompletionRelevanceData::Table(table);

        let detail: Option<String> = match table.table_kind {
            pgls_schema_cache::TableKind::Ordinary | pgls_schema_cache::TableKind::Partitioned => {
                None
            }
            pgls_schema_cache::TableKind::View => Some("View".into()),
            pgls_schema_cache::TableKind::MaterializedView => Some("MView".into()),
        };

        let item = PossibleCompletionItem {
            label: table.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: table.schema.to_string(),
            kind: CompletionItemKind::Table,
            detail,
            completion_text: Some(get_completion_text(ctx, table)),
        };

        builder.add_item(item);
    }
}

fn get_completion_text(ctx: &TreesitterContext, table: &Table) -> CompletionText {
    let text = with_schema_or_alias(ctx, table.name.as_str(), Some(table.schema.as_str()));

    let range = get_range_to_replace(ctx);

    CompletionText {
        text,
        range,
        is_snippet: false,
    }
}

#[cfg(test)]
mod tests {

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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn autocompletes_simple_table(pool: PgPool) {
        let setup = r#"
            create table users (
                id serial primary key,
                name text,
                password text
            );
        "#;

        let query = format!(
            "select * from u{}",
            QueryWithCursorPosition::cursor_marker()
        );

        let (tree, cache) = get_test_deps(Some(setup), query.as_str().into(), &pool).await;
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn autocompletes_table_alphanumerically(pool: PgPool) {
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

        pool.execute(setup).await.unwrap();

        let test_cases = vec![
            (
                format!(
                    "select * from u{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                "users",
            ),
            (
                format!(
                    "select * from e{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                "emails",
            ),
            (
                format!(
                    "select * from a{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                "addresses",
            ),
        ];

        for (query, expected_label) in test_cases {
            let (tree, cache) = get_test_deps(None, query.as_str().into(), &pool).await;
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn autocompletes_table_with_schema(pool: PgPool) {
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

        pool.execute(setup).await.unwrap();

        let test_cases = vec![
            (
                format!(
                    "select * from u{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                "user_y",
            ), // user_y is preferred alphanumerically
            (
                format!(
                    "select * from private.u{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                "user_z",
            ),
            (
                format!(
                    "select * from customer_support.u{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                "user_y",
            ),
        ];

        for (query, expected_label) in test_cases {
            let (tree, cache) = get_test_deps(None, query.as_str().into(), &pool).await;
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn prefers_table_in_from_clause(pool: PgPool) {
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

        let query = format!(
            r#"select * from coo{}"#,
            QueryWithCursorPosition::cursor_marker()
        );

        let (tree, cache) = get_test_deps(Some(setup), query.as_str().into(), &pool).await;
        let params = get_test_params(&tree, &cache, query.as_str().into());
        let items = complete(params);

        let CompletionItem { label, kind, .. } = items
            .into_iter()
            .next()
            .expect("Should return at least one completion item");

        assert_eq!(label, "coos");
        assert_eq!(kind, CompletionItemKind::Table);
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_update(pool: PgPool) {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!("update {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "public".into(),
                CompletionItemKind::Schema,
            )],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("update public.{}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "coos".into(),
                CompletionItemKind::Table,
            )],
            None,
            &pool,
        )
        .await;

        assert_no_complete_results(
            format!(
                "update public.coos {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "update coos set {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "update coos set name = 'cool' where {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_delete(pool: PgPool) {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );
        "#;

        pool.execute(setup).await.unwrap();

        assert_no_complete_results(
            format!("delete {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("delete from {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("coos".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "delete from public.{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::Label("coos".into())],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "delete from public.coos where {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_join(pool: PgPool) {
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

        assert_complete_results(
            format!(
                "select * from auth.users u join {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("posts".into(), CompletionItemKind::Table), // self-join
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            Some(setup),
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_alter_and_drop_statements(pool: PgPool) {
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
            format!("alter table {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("posts".into(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "alter table if exists {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("posts".into(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("drop table {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("posts".into(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "drop table if exists {}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("posts".into(), CompletionItemKind::Table), // self-join
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_insert_into(pool: PgPool) {
        let setup = r#"
            create schema auth;

            create table auth.users (
                uid serial primary key,
                name text not null,
                email text unique not null
            );
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!("insert into {}", QueryWithCursorPosition::cursor_marker()).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                "insert into auth.{}",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "users".into(),
                CompletionItemKind::Table,
            )],
            None,
            &pool,
        )
        .await;

        // works with complete statement.
        assert_complete_results(
            format!(
                "insert into {} (name, email) values ('jules', 'a@b.com');",
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn after_quoted_schemas(pool: PgPool) {
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
                r#"select * from "auth".{}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::CompletionTextAndRange(
                    "posts".into(),
                    TextRange::new(21.into(), 21.into()),
                ),
                CompletionAssertion::CompletionTextAndRange(
                    "users".into(),
                    TextRange::new(21.into(), 21.into()),
                ),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"select * from "auth"."{}""#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::CompletionTextAndRange(
                    "posts".into(),
                    TextRange::new(22.into(), 22.into()),
                ),
                CompletionAssertion::CompletionTextAndRange(
                    "users".into(),
                    TextRange::new(22.into(), 22.into()),
                ),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"select * from "auth"."{}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::CompletionTextAndRange(
                    r#"posts""#.into(),
                    TextRange::new(22.into(), 22.into()),
                ),
                CompletionAssertion::CompletionTextAndRange(
                    r#"users""#.into(),
                    TextRange::new(22.into(), 22.into()),
                ),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_tables_in_policies(pool: PgPool) {
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
                r#"create policy "my cool pol" on {}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("auth".into(), CompletionItemKind::Schema),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!(
                r#"create policy "my cool pol" on auth.{}"#,
                QueryWithCursorPosition::cursor_marker()
            )
            .as_str(),
            vec![
                CompletionAssertion::LabelAndKind("posts".into(), CompletionItemKind::Table),
                CompletionAssertion::LabelAndKind("users".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;
    }
}
