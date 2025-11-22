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
    use pgls_test_utils::QueryWithCursorPosition;
    use sqlx::PgPool;

    use crate::test_helper::{
        TestCompletionsCase, TestCompletionsSuite, assert_complete_results,
        assert_no_complete_results,
    };

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn handles_nested_queries(pool: PgPool) {
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

        TestCompletionsSuite::new(&pool, Some(setup)).with_case(
        TestCompletionsCase::new()
        .inside_static_statement(r#"
            select * from (
                <sql>
            ) as subquery
            join public.users u
            on u.id = subquery.id;
            "#)
            .type_sql("select id, narrator_id<1> from private.audio_books")
            .comment("Should prefer the one from private.audio_audiobooks, since the other tables are out of scope.")
        )
            .snapshot("handles_nested_queries")
            .await;
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("select narrator_id<1>")
                    .comment("Should suggest all columns with n first"),
            )
            .snapshot("shows_multiple_columns_if_no_relation_specified")
            .await;
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(TestCompletionsCase::new().type_sql("select name from users"))
            .snapshot("suggests_relevant_columns_without_letters")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn ignores_cols_in_from_clause(pool: PgPool) {
        let setup = r#"
        create schema private;

        create table private.users (
            id serial primary key,
            name text,
            address text,
            email text,
            public boolean
        );
    "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("select * from <sql>")
                    .type_sql("private<1>.users")
                    .comment("No column suggestions."),
            )
            .snapshot("ignores_cols_in_from_clause")
            .await;
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("<sql> from public.users")
                    .type_sql("select address2<1>")
                    .comment("Should suggest address 2 from public table"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("<sql> from private.users")
                    .type_sql("select address1<1>")
                    .comment("Should suggest address 1 from private table"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement("<sql> from private.users")
                    .type_sql("select settings<1>")
                    .comment("Should prioritize columns starting with s"),
            )
            .snapshot("prefers_columns_of_mentioned_tables")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn filters_out_by_aliases_in_join_on(pool: PgPool) {
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        "select u.id, p.content from auth.users u join auth.posts p <sql>",
                    )
                    .type_sql("on u<1>.id = p.<2>user_id")
                    .comment("Should prefer primary indices here.")
                    .comment("We should only get columns from the auth.posts table."),
            )
            .snapshot("filters_out_by_aliases_in_join_on")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn filters_out_by_aliases_in_select(pool: PgPool) {
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        "<sql> from auth.users u join auth.posts p on u.id = p.user_id;",
                    )
                    .type_sql("select u.id, p.pid<1>")
                    .comment("We should only get columns from the auth.posts table."),
            )
            .snapshot("filters_out_by_aliases_in_select")
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

        TestCompletionsSuite::new(&pool, Some(setup)).with_case(
        TestCompletionsCase::new()
            .type_sql("select u.uid, p.content from auth<1>.users<2> u join auth.posts p on u.uid = p.user_id")
            .comment("Schema suggestions should be prioritized, since we want to push users to specify them.")
            .comment("Here, we shouldn't have schema completions.")
        ).snapshot("does_not_complete_cols_in_join_clauses").await;
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

        TestCompletionsSuite::new(&pool, Some(setup)).with_case(
            TestCompletionsCase::new()
                .inside_static_statement(
                    "select u.id, auth.posts.content from auth.users u join auth.posts p on <sql>",
                )
                .type_sql("<1>p.user_id<2> = u.uid<3>;")
                .comment("Should prioritize primary keys here.")
                .comment("Should only consider columns from auth.posts here.")
                .comment("Should only consider columns from auth.users here.")
        ).snapshot("completes_in_join_on_clause").await;
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        "<sql> from public.one o join public.two on o.id = t.id;",
                    )
                    .type_sql("select o.id, a, <1>b, c, d, e, <2>z")
                    .comment("Should have low priority for `a`, since it's already mentioned.")
                    .comment("Should have high priority of id of table two, but not one, since it's already mentioned.")
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        "<sql> from public.one o join public.two on o.id = t.id;",
                    )
                    .type_sql("select id, a, b, c, d, e, <1>z")
                    .comment("`id` could be from both tables, so both priorities are lowered."),
            )
            .snapshot("prefers_not_mentioned_columns")
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("insert into instruments (id, name) values (1, 'my_bass');"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .type_sql(r#"insert into instruments ("id", "name") values (1, 'my_bass');"#),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        r#"insert into instruments (<sql>, name) values ('my_bass');"#,
                    )
                    .type_sql("id, <1>z")
                    .comment("`name` is already written, so z should be suggested."),
            )
            .snapshot("suggests_columns_in_insert_clause")
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        "select name from instruments i join others o on i.z = o.a <sql>",
                    )
                    .type_sql("where o.<1>a = <2>i.z and <3>i.id > 5;")
                .comment("should respect alias speciifcation")
                .comment("should not prioritize suggest columns or schemas (right side of binary expression)")
                .comment("should prioritize columns that aren't already mentioned")
            )
            .snapshot("suggests_columns_in_where_clause")
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
        "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new().type_sql("alter table instruments drop column name"),
            )
            .with_case(
                TestCompletionsCase::new().type_sql("alter table instruments drop column name"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("alter table instruments drop column if exists name"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("alter table instruments alter column name set default"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("alter table instruments alter name set default"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("alter table public.instruments alter column name"),
            )
            .with_case(TestCompletionsCase::new().type_sql("alter table instruments alter name"))
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("alter table instruments rename name to new_col"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .type_sql("alter table public.instruments rename column name to new_col"),
            )
            .snapshot("suggests_columns_in_alter_table_and_drop_table")
            .await;
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        r#"create policy "my_pol" on public.instruments for select using (<sql>)"#,
                    )
                    .type_sql("id = 1 and created_at > '2025-01-01'"),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        r#"create policy "my_pol" on public.instruments for insert with check (<sql>)"#,
                    )
                    .type_sql("id = 1 and created_at > '2025-01-01'"),
            )
            .snapshot("suggests_columns_policy_using_clause")
            .await;
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new().type_sql(r#"select "email" from "private"."users";"#),
            )
            .snapshot("completes_quoted_columns")
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

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(
                TestCompletionsCase::new()
                    .type_sql(r#"select "pr"."email" from private.users "pr""#),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(r#"<sql> from private.users "pr""#)
                    .type_sql(r#"select "email""#),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(r#"<sql> from private.users "pr""#)
                    .type_sql(r#"select pr."email""#),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(r#"<sql> from private.users "pr""#)
                    .type_sql(r#"select "pr"."email""#),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(r#"<sql> from private.users "pr""#)
                    .type_sql(r#"select pr.<1>email"#)
                    .comment("not quoted here, since the alias isn't."),
            )
            .with_case(
                TestCompletionsCase::new()
                    .inside_static_statement(
                        r#"<sql> from private.users "pr" join public.names n on pr.id = n.uid;"#,
                    )
                    .type_sql(r#"select "pr"."email", n.uid"#),
            )
            .snapshot("completes_quoted_columns_with_aliases")
            .await;
    }
}
