use crate::{
    builder::{CompletionBuilder, PossibleCompletionItem},
    context::CompletionContext,
    item::CompletionItemKind,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::get_completion_text_with_schema_or_alias;

pub fn complete_tables<'a>(ctx: &'a CompletionContext, builder: &mut CompletionBuilder<'a>) {
    let available_tables = &ctx.schema_cache.tables;

    for table in available_tables {
        let relevance = CompletionRelevanceData::Table(table);

        let detail: Option<String> = match table.table_kind {
            pgt_schema_cache::TableKind::Ordinary | pgt_schema_cache::TableKind::Partitioned => {
                None
            }
            pgt_schema_cache::TableKind::View => Some("View".into()),
            pgt_schema_cache::TableKind::MaterializedView => Some("MView".into()),
        };

        let item = PossibleCompletionItem {
            label: table.name.clone(),
            score: CompletionScore::from(relevance.clone()),
            filter: CompletionFilter::from(relevance),
            description: table.schema.to_string(),
            kind: CompletionItemKind::Table,
            detail,
            completion_text: get_completion_text_with_schema_or_alias(
                ctx,
                &table.name,
                &table.schema,
            ),
        };

        builder.add_item(item);
    }
}

#[cfg(test)]
mod tests {

    use sqlx::{Executor, PgPool};

    use crate::{
        CompletionItem, CompletionItemKind, complete,
        test_helper::{
            CURSOR_POS, CompletionAssertion, assert_complete_results, assert_no_complete_results,
            get_test_deps, get_test_params,
        },
    };

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn autocompletes_simple_table(pool: PgPool) {
        let setup = r#"
            create table users (
                id serial primary key,
                name text,
                password text
            );
        "#;

        let query = format!("select * from u{}", CURSOR_POS);

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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
            (format!("select * from u{}", CURSOR_POS), "users"),
            (format!("select * from e{}", CURSOR_POS), "emails"),
            (format!("select * from a{}", CURSOR_POS), "addresses"),
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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
            (format!("select * from u{}", CURSOR_POS), "user_y"), // user_y is preferred alphanumerically
            (format!("select * from private.u{}", CURSOR_POS), "user_z"),
            (
                format!("select * from customer_support.u{}", CURSOR_POS),
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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

        let query = format!(r#"select * from coo{}"#, CURSOR_POS);

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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_update(pool: PgPool) {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );
        "#;

        pool.execute(setup).await.unwrap();

        assert_complete_results(
            format!("update {}", CURSOR_POS).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "public".into(),
                CompletionItemKind::Schema,
            )],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("update public.{}", CURSOR_POS).as_str(),
            vec![CompletionAssertion::LabelAndKind(
                "coos".into(),
                CompletionItemKind::Table,
            )],
            None,
            &pool,
        )
        .await;

        assert_no_complete_results(
            format!("update public.coos {}", CURSOR_POS).as_str(),
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("update coos set {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("update coos set name = 'cool' where {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn suggests_tables_in_delete(pool: PgPool) {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );
        "#;

        pool.execute(setup).await.unwrap();

        assert_no_complete_results(format!("delete {}", CURSOR_POS).as_str(), None, &pool).await;

        assert_complete_results(
            format!("delete from {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::LabelAndKind("public".into(), CompletionItemKind::Schema),
                CompletionAssertion::LabelAndKind("coos".into(), CompletionItemKind::Table),
            ],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("delete from public.{}", CURSOR_POS).as_str(),
            vec![CompletionAssertion::Label("coos".into())],
            None,
            &pool,
        )
        .await;

        assert_complete_results(
            format!("delete from public.coos where {}", CURSOR_POS).as_str(),
            vec![
                CompletionAssertion::Label("id".into()),
                CompletionAssertion::Label("name".into()),
            ],
            None,
            &pool,
        )
        .await;
    }

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
            format!("select * from auth.users u join {}", CURSOR_POS).as_str(),
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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
            format!("alter table {}", CURSOR_POS).as_str(),
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
            format!("alter table if exists {}", CURSOR_POS).as_str(),
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
            format!("drop table {}", CURSOR_POS).as_str(),
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
            format!("drop table if exists {}", CURSOR_POS).as_str(),
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

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
            format!("insert into {}", CURSOR_POS).as_str(),
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
            format!("insert into auth.{}", CURSOR_POS).as_str(),
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
                CURSOR_POS
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
}
