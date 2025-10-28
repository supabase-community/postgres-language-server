use pgls_hover::{OnHoverParams, on_hover};
use pgls_schema_cache::SchemaCache;
use pgls_test_utils::QueryWithCursorPosition;
use pgls_text_size::TextSize;
use sqlx::{Executor, PgPool};

async fn test_hover_at_cursor(name: &str, query: String, setup: Option<&str>, test_db: &PgPool) {
    if let Some(setup) = setup {
        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");
    }

    let schema_cache = SchemaCache::load(test_db)
        .await
        .expect("Failed to load Schema Cache");

    let (position, sql) = QueryWithCursorPosition::from(query).get_text_and_position();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .expect("Error loading sql language");

    let tree = parser.parse(&sql, None).unwrap();
    let ast = pgls_query::parse(&sql)
        .ok()
        .map(|parsed| parsed.into_root().unwrap());

    let hover_results = on_hover(OnHoverParams {
        position: TextSize::new(position as u32),
        schema_cache: &schema_cache,
        stmt_sql: &sql,
        ast: ast.as_ref(),
        ts_tree: &tree,
    });

    let mut snapshot = String::new();
    snapshot.push_str("# Input\n");
    snapshot.push_str("```sql\n");
    snapshot.push_str(&sql);
    snapshot.push('\n');

    for _ in 0..position {
        snapshot.push(' ');
    }
    snapshot.push_str("↑ hovered here\n");

    snapshot.push_str("```\n\n");

    if hover_results.is_empty() {
        snapshot.push_str("# Hover Results\n");
        snapshot.push_str("No hover information found.\n");
    } else {
        snapshot.push_str("# Hover Results\n");
        for (i, result) in hover_results.iter().enumerate() {
            if i > 0 {
                snapshot.push_str("\n---\n\n");
            }
            snapshot.push_str(result);
            snapshot.push('\n');
        }
    }

    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, snapshot);
    });
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_builtin_count(test_db: PgPool) {
    let query = format!(
        "select cou{}nt(*) from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("builtin_count", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_builtin_now(test_db: PgPool) {
    let query = format!(
        "select n{}ow() from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("builtin_now", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_builtin_max(test_db: PgPool) {
    let query = format!(
        "select m{}ax(id) from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("builtin_max", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_custom_function(test_db: PgPool) {
    let setup = r#"
        create or replace function custom_add(a integer, b integer)
        returns integer
        language plpgsql
        immutable
        as $$
        begin
            return a + b;
        end;
        $$;
    "#;

    let query = format!(
        "select custom_a{}dd(1, 2)",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("custom_function", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_with_schema(test_db: PgPool) {
    let setup = r#"
        create schema test_schema;
        create or replace function test_schema.schema_func(text)
        returns text
        language sql
        stable
        as $$
            select $1 || ' processed';
        $$;
    "#;

    let query = format!(
        "select test_schema.schema_f{}unc('test')",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("function_with_schema", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_with_table_ref(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        "select users.i{}d from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("column_hover", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_in_join(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            email varchar(255) not null
        );

        create table posts (
            id serial primary key,
            user_id serial references users(id),
            content text
        );
    "#;

    let query = format!(
        "select * from users u join posts p on u.id = p.use{}r_id",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("column_hover_join", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_works(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        "select id from use{}rs",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("table_hover", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_no_hover_on_keyword(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        "sel{}ect id from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("no_hover_keyword", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn shortens_lengthy_functions(test_db: PgPool) {
    let setup = r#"
        create or replace function public.func(cool_stuff text,       something_else int,a_third_thing text)
        returns void
        language sql
        stable
        as $$
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
            select 1;
        $$;
    "#;

    let query = format!(
        "select public.fu{}nc()",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("lenghty_function", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_role_hover_create_role(test_db: PgPool) {
    let query = format!(
        "create role alternate_ow{}ner with superuser createdb login",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("role_create", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_role_hover_alter_role(test_db: PgPool) {
    let query = format!(
        "alter role test_log{}in set work_mem = '256MB'",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("role_alter", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_with_quoted_schema(test_db: PgPool) {
    let setup = r#"
        create schema auth;
        create table auth.users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        r#"select * from "auth".use{}rs"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("table_hover_quoted_schema", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_with_quoted_schema(test_db: PgPool) {
    let setup = r#"
        create schema auth;
        create or replace function auth.authenticate_user(user_email text)
        returns boolean
        language plpgsql
        security definer
        as $$
        begin
            return exists(select 1 from auth.users where email = user_email);
        end;
        $$;
    "#;

    let query = format!(
        r#"select "auth".authenticate_u{}ser('test@example.com')"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("function_hover_quoted_schema", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_with_quoted_schema_table(test_db: PgPool) {
    let setup = r#"
        create schema auth;
        create table auth.user_profiles (
            id serial primary key,
            user_id int not null,
            first_name varchar(100),
            last_name varchar(100)
        );
    "#;

    let query = format!(
        r#"select "auth"."user_profiles".first_n{}ame from "auth"."user_profiles""#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "column_hover_quoted_schema_table",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_with_quoted_table_name(test_db: PgPool) {
    let setup = r#"
        create schema auth;
        create table auth.users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        r#"select * from "auth"."use{}rs""#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "table_hover_quoted_table_name",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_with_quoted_column_name(test_db: PgPool) {
    let setup = r#"
        create schema auth;
        create table auth.users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        r#"select "ema{}il" from auth.users"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "column_hover_quoted_column_name",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_with_quoted_column_name_with_table(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            email varchar(255) not null
        );

        create table phone_nums (
            phone_id serial primary key,
            email varchar(255) not null,
            phone int
        );
    "#;

    let query = format!(
        r#"select phone, id from users join phone_nums on "users"."em{}ail" = phone_nums.email;"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "column_hover_quoted_column_name_with_table",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn hover_on_schemas(test_db: PgPool) {
    let setup = r#"
        create schema auth;

        create table auth.users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        r#"select * from au{}th.users;"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("hover_on_schemas", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_policy_table_hover(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            name text
        );
    "#;

    test_db.execute(setup).await.unwrap();

    let query = format!(
        r#"create policy "my cool pol" on us{}ers for all to public with check (true);"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("create_policy", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_revoke_table_hover(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            name text
        );
    "#;

    test_db.execute(setup).await.unwrap();

    let query = format!(
        "revoke select on us{}ers from public;",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("revoke_select", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_grant_table_hover(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            name text
        );
    "#;

    test_db.execute(setup).await.unwrap();

    let query = format!(
        "grant select on us{}ers to public;",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("grant_select", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn hover_on_composite_type(test_db: PgPool) {
    let setup = r#"create type compfoo as (f1 int, f2 text);"#;

    let query = format!(
        "create function getfoo() returns setof comp{}foo as $$ select fooid, fooname from foo $$ language sql;",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "hover_custom_type_with_properties",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn hover_on_enum_type(test_db: PgPool) {
    let setup = r#"create type compfoo as ENUM ('yes', 'no');"#;

    let query = format!(
        "create function getfoo() returns setof comp{}foo as $$ select fooid, fooname from foo $$ language sql;",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("hover_custom_type_enum", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn hover_type_in_select_clause(test_db: PgPool) {
    let setup = r#"create type compfoo as (f1 int, f2 text);"#;

    let query = format!(
        "select (co{}mpfoo).f1 from some_table s;",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("hover_type_in_select_clause", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn no_hover_results_over_params(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            name text
        );
    "#;

    test_db.execute(setup).await.unwrap();

    {
        let query = format!(
            "select * from users where name = $n{}ame;",
            QueryWithCursorPosition::cursor_marker()
        );
        test_hover_at_cursor("dollar-param", query, None, &test_db).await;
    }
    {
        let query = format!(
            "select * from users where name = :n{}ame;",
            QueryWithCursorPosition::cursor_marker()
        );
        test_hover_at_cursor("colon-param", query, None, &test_db).await;
    }
    {
        let query = format!(
            "select * from users where name = @n{}ame;",
            QueryWithCursorPosition::cursor_marker()
        );
        test_hover_at_cursor("at-param", query, None, &test_db).await;
    }
    {
        let query = format!(
            "select * from users where name = ?n{}ame;",
            QueryWithCursorPosition::cursor_marker()
        );
        test_hover_at_cursor("questionmark-param", query, None, &test_db).await;
    }
}
