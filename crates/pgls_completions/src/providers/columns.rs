use pgls_catalog::{Column, NamePosition, Snapshot, identifier_name};
use pgls_treesitter::TreesitterContext;

use crate::{
    CompletionItemKind, CompletionText,
    builder::{CompletionBuilder, PossibleCompletionItem},
    providers::helper::get_range_to_replace,
    relevance::{CompletionRelevanceData, filtering::CompletionFilter, scoring::CompletionScore},
};

use super::helper::{Qualifier, with_schema_or_alias};

pub fn complete_columns<'a>(
    ctx: &TreesitterContext<'a>,
    snapshot: &'a Snapshot,
    builder: &mut CompletionBuilder<'a>,
) {
    let available_columns = &snapshot.columns;

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
    // A `column_identifier` is a column name that can't be qualified, like the target of
    // `SET col = ...` or a column of an INSERT column list. The alias is a name, which is quoted
    // again if needed.
    let alias = (ctx.node_under_cursor.kind() != "column_identifier")
        .then(|| ctx.get_used_alias_for_table(&col.schema_name, &col.table_name))
        .flatten()
        .map(|alias| identifier_name(&alias));

    let with_schema_or_alias = with_schema_or_alias(
        ctx,
        col.name.as_str(),
        alias.as_deref().map(Qualifier::Alias),
        NamePosition::ColId,
    );

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
    use sqlx::{Executor, PgPool};

    use crate::{
        CompletionItemKind, complete,
        test_helper::{
            CompletionAssertion, TestCompletionsCase, TestCompletionsSuite,
            assert_complete_results, get_test_deps, get_test_params,
        },
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
                <sql> from private.audio_books
            ) as subquery
            join public.users u
            on u.id = subquery.id;
            "#)
            .type_sql("select id, narrator_id<1>")
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
                    .comment("should respect alias specification")
                    .comment("should not prioritize suggest columns or schemas (right side of binary expression)")
                    .comment("should prioritize columns that aren't already mentioned"),
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_the_columns_of_the_table_an_identifier_names(pool: PgPool) {
        let setup = r#"
            create table "MyTable" (quoted_id int);
            create table mytable (folded_id int);
            create table "a""b" (escaped_id int);
        "#;
        pool.execute(setup).await.unwrap();

        // Unquoted names are downcased, quoted names keep their case and unescape `""`.
        for (table, column) in [
            (r#""MyTable""#, "quoted_id"),
            ("MyTable", "folded_id"),
            (r#""a""b""#, "escaped_id"),
        ] {
            let others = ["quoted_id", "folded_id", "escaped_id"]
                .into_iter()
                .filter(|other| *other != column)
                .map(|other| CompletionAssertion::LabelNotExists(other.into()));
            assert_complete_results(
                format!(
                    "select t.{} from {table} t",
                    QueryWithCursorPosition::cursor_marker()
                )
                .as_str(),
                std::iter::once(CompletionAssertion::Label(column.into()))
                    .chain(others)
                    .collect(),
                None,
                &pool,
            )
            .await;
        }
    }

    /// The labels of the columns offered at the cursor, best first, on a Postgres 18 server.
    async fn columns_at(query: &str, pool: &PgPool) -> Vec<String> {
        columns_on(180000, query, pool).await
    }

    /// The labels of the columns offered at the cursor, best first, as if the server had the
    /// version `version_num`. The test database can be older than the syntax under test.
    async fn columns_on(version_num: i64, query: &str, pool: &PgPool) -> Vec<String> {
        let (tree, mut snapshot) = get_test_deps(None, query.into(), pool).await;
        snapshot.version.version_num = Some(version_num);
        let params = get_test_params(&tree, &snapshot, query.into());
        complete(params)
            .into_iter()
            .filter(|item| item.kind == CompletionItemKind::Column)
            .map(|item| item.label)
            .collect()
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_columns_after_opening_parenthesis(pool: PgPool) {
        pool.execute("create table public.users (id serial primary key, name text);")
            .await
            .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        for sql in [
            "select ({}) from users",
            "select * from users where ({})",
            "insert into users ({})",
            // Without the closing parenthesis.
            "insert into users ({}",
        ] {
            let query = sql.replace("{}", &cursor.to_string());
            let mut columns = columns_at(&query, &pool).await;
            columns.truncate(2);
            columns.sort();
            assert_eq!(columns, ["id", "name"], "{sql}");
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_the_columns_of_update_and_delete_aliases(pool: PgPool) {
        pool.execute(
            r#"
            create table public.users (id serial primary key, name text);
            create table public.posts (post_id int, title text);
            "#,
        )
        .await
        .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        for (sql, expected) in [
            ("update users u set name = 'x' where u.{}", ["id", "name"]),
            (
                "update users as u set name = 'x' returning u.{}",
                ["id", "name"],
            ),
            ("delete from users u where u.{}", ["id", "name"]),
            ("delete from users as u returning u.{}", ["id", "name"]),
            (
                "delete from users u using posts p where p.{}",
                ["post_id", "title"],
            ),
        ] {
            let query = sql.replace("{}", &cursor.to_string());
            let mut columns = columns_at(&query, &pool).await;
            columns.sort();
            assert_eq!(columns, expected, "{sql}");
        }
    }

    /// The source after applying the completion of the column `label` at the cursor.
    async fn apply_column(query: &str, label: &str, pool: &PgPool) -> String {
        let (tree, snapshot) = get_test_deps(None, query.into(), pool).await;
        let params = get_test_params(&tree, &snapshot, query.into());
        let (_, mut source) = QueryWithCursorPosition::from(query).get_text_and_position();
        let item = complete(params)
            .into_iter()
            .find(|item| item.kind == CompletionItemKind::Column && item.label == label)
            .unwrap_or_else(|| panic!("no column {label} in {query}"));
        let edit = item.completion_text.unwrap();
        source.replace_range(std::ops::Range::<usize>::from(edit.range), &edit.text);
        source
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn qualifies_columns_with_the_alias_of_their_schema(pool: PgPool) {
        pool.execute(
            r#"
            create schema auth;
            create table public.users (name text);
            create table auth.users (auth_only int);
            "#,
        )
        .await
        .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        // Both aliases stand for a table named `users`; the aliases are kept in a hash map, so
        // a lookup by the table name alone picks either.
        let query = format!(
            "update public.users u set name = 1 from auth.users p where auth_{cursor}only = 1"
        );
        for _ in 0..20 {
            assert_eq!(
                apply_column(&query, "auth_only", &pool).await,
                "update public.users u set name = 1 from auth.users p where p.auth_only = 1"
            );
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn keeps_escaped_quotes_of_aliases(pool: PgPool) {
        pool.execute("create table public.users (id int, name text);")
            .await
            .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        assert_eq!(
            apply_column(
                &format!(r#"update users as "a""b" set name = 1 where "i{cursor}d" = 1"#),
                "id",
                &pool
            )
            .await,
            r#"update users as "a""b" set name = 1 where "a""b"."id" = 1"#
        );
        assert_eq!(
            apply_column(
                &format!(r#"select "{cursor} from users as "a""b""#),
                "id",
                &pool
            )
            .await,
            r#"select "a""b"."id" from users as "a""b""#
        );
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_columns_of_old_and_new_in_returning(pool: PgPool) {
        pool.execute(
            r#"
            create table public.users (id serial primary key, name text);
            create table public.posts (post_id int, title text);
            "#,
        )
        .await
        .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        for sql in [
            "update users set name = 'x' returning old.{}",
            "update users set name = 'x' returning new.{}",
            "insert into users (name) values ('x') returning new.{}",
            "delete from users returning old.{}",
            "merge into users u using posts p on u.id = p.post_id when matched then delete returning old.{}",
            "with d as (delete from users returning old.{}) select * from d",
            // `WITH (...)` renames them.
            "update users set name = 'x' returning with (old as o, new as \"N\") \"N\".id, o.{}",
            // A FROM item named `new` doesn't hide a name given in `WITH (...)`.
            "update users set name = 'x' from posts as new returning with (new as new) new.{}",
        ] {
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(columns_at(&query, &pool).await, ["id", "name"], "{sql}");
        }

        assert_eq!(
            columns_at(
                &format!("delete from users returning old.na{cursor}"),
                &pool
            )
            .await,
            ["name"]
        );
        // A FROM item named `old` hides the default name.
        assert_eq!(
            columns_at(
                &format!("update users set name = 'x' from posts as old returning old.{cursor}"),
                &pool
            )
            .await,
            ["post_id", "title"]
        );
        // `old` is renamed, and outside RETURNING there is no `old`.
        for sql in [
            "update users set name = 'x' returning with (old as o) old.{}",
            "update users set name = 'x' where old.{} returning old.id",
        ] {
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(columns_at(&query, &pool).await, [""; 0], "{sql}");
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn no_old_and_new_before_postgres_18(pool: PgPool) {
        pool.execute("create table public.users (id serial primary key, name text);")
            .await
            .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        for sql in [
            "update users set name = 'x' returning old.{}",
            "update users set name = 'x' returning with (old as o) o.{}",
        ] {
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(columns_on(170000, &query, &pool).await, [""; 0], "{sql}");
        }
        // The target's columns are still offered.
        let mut columns = columns_on(
            170000,
            &format!("update users set name = 'x' returning {cursor}"),
            &pool,
        )
        .await;
        columns.truncate(2);
        columns.sort();
        assert_eq!(columns, ["id", "name"]);
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_columns_in_returning(pool: PgPool) {
        pool.execute(
            r#"
            create table public.users (id serial primary key, name text);
            create table public.posts (post_id int, title text);
            "#,
        )
        .await
        .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();

        // The columns of the target come first.
        let mut columns = columns_at(
            &format!("update users set name = 'x' returning {cursor}"),
            &pool,
        )
        .await;
        columns.truncate(2);
        columns.sort();
        assert_eq!(columns, ["id", "name"]);

        // No columns while `WITH (...)` is typed.
        for sql in [
            "update users set name = 'x' returning with ({})",
            "update users set name = 'x' returning with (old {})",
            "update users set name = 'x' returning with (old as {})",
        ] {
            let query = sql.replace("{}", &cursor.to_string());
            assert_eq!(columns_at(&query, &pool).await, [""; 0], "{sql}");
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_in_merge(pool: PgPool) {
        let setup = r#"
            create table public.users (id serial primary key, name text);
            create table public.posts (post_id int, title text);
        "#;

        TestCompletionsSuite::new(&pool, Some(setup))
            .with_case(TestCompletionsCase::new().type_sql(
                "merge into users u using posts p on p.post_id = u.id \
                 when matched and p.title <> u.name then update set name = p.title \
                 when not matched then insert (id, name) values (p.post_id, p.title);",
            ))
            .snapshot("completes_in_merge")
            .await;
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_the_columns_of_merge_aliases(pool: PgPool) {
        pool.execute(
            r#"
            create table public.users (id serial primary key, name text);
            create table public.posts (post_id int, title text);
            "#,
        )
        .await
        .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();
        let merge = "merge into users u using posts as p on p.post_id = u.id";

        for (sql, columns) in [
            (
                "merge into users u using posts p on p.{}",
                &["post_id", "title"][..],
            ),
            (
                "merge into users u using posts p on p.post_id = u.{}",
                &["id", "name"],
            ),
            ("{merge} when matched and u.{} then delete", &["id", "name"]),
            (
                "{merge} when matched then update set name = p.{}",
                &["post_id", "title"],
            ),
            (
                "{merge} when not matched then insert (id) values (p.{})",
                &["post_id", "title"],
            ),
            (
                "{merge} when matched then delete returning p.{}",
                &["post_id", "title"],
            ),
        ] {
            let query = sql
                .replace("{merge}", merge)
                .replace("{}", &cursor.to_string());
            assert_eq!(columns_at(&query, &pool).await, columns, "{sql}");
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_the_columns_each_merge_branch_sees(pool: PgPool) {
        pool.execute(
            r#"
            create table public.tgt (tgt_id int, tgt_label text);
            create table public.src (src_id int, src_label text);
            "#,
        )
        .await
        .unwrap();
        let cursor = QueryWithCursorPosition::cursor_marker();
        let merge = "merge into tgt t using src s on s.src_id = t.tgt_id";
        let target = &["tgt_id", "tgt_label"][..];
        let source = &["src_id", "src_label"][..];
        let both = &["src_id", "src_label", "tgt_id", "tgt_label"][..];
        let none = &[][..];

        for (sql, expected) in [
            ("{merge} when matched and {} then delete", both),
            ("{merge} when matched then update set tgt_label = {}", both),
            // `NOT MATCHED [BY TARGET]` only sees the source.
            ("{merge} when not matched then insert values ({})", source),
            (
                "{merge} when not matched by target and {} then do nothing",
                source,
            ),
            ("{merge} when not matched then insert values (t.{})", none),
            // `NOT MATCHED BY SOURCE` only sees the target.
            (
                "{merge} when not matched by source and {} then delete",
                target,
            ),
            (
                "{merge} when not matched by source then update set tgt_label = s.{}",
                none,
            ),
            // The target columns of INSERT and UPDATE.
            ("{merge} when not matched then insert ({})", target),
            ("{merge} when matched then update set {}", target),
            // The same table as target and source stays visible. Columns of tables the
            // statement doesn't mention are offered anyway, ranked lower.
            (
                "merge into tgt a using tgt b on a.tgt_id = b.tgt_id when not matched then insert values ({})",
                both,
            ),
        ] {
            let query = sql
                .replace("{merge}", merge)
                .replace("{}", &cursor.to_string());
            let mut columns = columns_at(&query, &pool).await;
            columns.retain(|column| column.starts_with("tgt_") || column.starts_with("src_"));
            columns.sort();
            assert_eq!(columns, expected, "{sql}");
        }
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn completes_a_column_named_period_in_a_key(pool: PgPool) {
        pool.execute("create table public.t (period int);")
            .await
            .unwrap();

        // `period` is an unreserved keyword that may come before the column of a temporal key,
        // and here it is the column itself.
        let cursor = QueryWithCursorPosition::cursor_marker();
        for (before, after) in [("per", "iod"), ("period", ""), ("", "period")] {
            for query in [
                format!("create table t (period int, primary key ({before}{cursor}{after}));"),
                format!(
                    "create table u (period int, foreign key (period) references t ({before}{cursor}{after}));"
                ),
            ] {
                let (tree, snapshot) = get_test_deps(None, query.as_str().into(), &pool).await;
                let items = complete(get_test_params(&tree, &snapshot, query.as_str().into()));

                let item = items
                    .iter()
                    .find(|item| item.label == "period")
                    .unwrap_or_else(|| panic!("no completion for `period` in {query}"));
                assert_eq!(item.kind, CompletionItemKind::Column, "{query}");
                assert_eq!(item.description, "public.t", "{query}");
                assert_eq!(item.detail.as_deref(), Some("int4"), "{query}");
            }
        }
    }
}
