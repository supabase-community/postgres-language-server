use std::sync::Arc;

use pgls_catalog::resolve::{ResolveParams, resolve};
use pgls_catalog::{Catalog, CatalogBase, Snapshot};
use pgls_hover::{OnHoverParams, on_hover};
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

    let snapshot = Arc::new(
        Snapshot::load(test_db)
            .await
            .expect("Failed to load snapshot"),
    );

    let (position, sql) = QueryWithCursorPosition::from(query).get_text_and_position();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .expect("Error loading sql language");

    let tree = parser.parse(&sql, None).unwrap();
    let ast = pgls_query::parse(&sql)
        .ok()
        .map(|parsed| parsed.into_root().unwrap());

    // What the workspace passes: the references at the cursor.
    let position = TextSize::new(position as u32);
    let references = ast
        .as_ref()
        .map(|stmt| {
            let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(Arc::clone(&snapshot)))));
            resolve(ResolveParams {
                stmt,
                catalog: &catalog,
                search_path: &snapshot.expand_search_path(&["public".to_owned()]),
                function: None,
                sql: Some(&sql),
                collect_references: true,
            })
            .references
        })
        .unwrap_or_default()
        .into_iter()
        .filter(|reference| {
            reference
                .span()
                .is_some_and(|span| span.contains_inclusive(position))
        })
        .collect::<Vec<_>>();

    let hover_results = on_hover(OnHoverParams {
        position,
        snapshot: &snapshot,
        stmt_sql: &sql,
        ast: ast.as_ref(),
        ts_tree: &tree,
        references: &references,
    })
    .unwrap_or_default();

    let mut snapshot = String::new();
    snapshot.push_str("# Input\n");
    snapshot.push_str("```sql\n");
    snapshot.push_str(&sql);
    snapshot.push('\n');

    for _ in 0..usize::from(position) {
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
async fn test_table_hover_select_star(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            email varchar(255) not null
        );
    "#;

    let query = format!(
        "select * from use{}rs",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("table_hover_select_star", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_shows_nullable_and_basic_defaults(test_db: PgPool) {
    let setup = r#"
        create table users (
            id serial primary key,
            name text default 'anonymous',
            score int default 0,
            enabled bool default false,
            created_at timestamptz default now()
        );
    "#;

    let query = format!(
        "select * from use{}rs",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "table_hover_nullable_and_basic_defaults",
        query,
        Some(setup),
        &test_db,
    )
    .await;
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

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_overload(test_db: PgPool) {
    let setup = r#"
        create function pick(i integer) returns integer language sql immutable as 'select i';
        create function pick(t text) returns text language sql immutable as 'select t';
    "#;

    let query = format!("select pi{}ck(1)", QueryWithCursorPosition::cursor_marker());

    test_hover_at_cursor("function_overload", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_function_hover_ambiguous_overload_falls_back(test_db: PgPool) {
    let setup = r#"
        create function amb(a integer, b text) returns integer language sql immutable as 'select 1';
        create function amb(c text, d integer) returns integer language sql immutable as 'select 1';
    "#;

    let query = format!(
        "select am{}b('1', '2')",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("function_ambiguous_overload", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_resolved_alias_in_join(test_db: PgPool) {
    let setup = r#"
        create table hover_users (id serial primary key, name text not null);
        create table hover_orgs (id serial primary key, name text);
    "#;

    let query = format!(
        "select o.na{}me from hover_users u join hover_orgs o on o.id = u.id",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "column_resolved_alias_in_join",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_resolved_unqualified_in_join(test_db: PgPool) {
    let setup = r#"
        create table hover_users (id serial primary key, name text not null);
        create table hover_orgs (id serial primary key, name text, slug text);
    "#;

    let query = format!(
        "select sl{}ug from hover_users u join hover_orgs o on o.id = u.id",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "column_resolved_unqualified_in_join",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_cte_column(test_db: PgPool) {
    let setup = r#"
        create table hover_users (id serial primary key, name text not null);
    "#;

    let query = format!(
        "with c as (select id, name as label from hover_users) select lab{}el from c",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("column_cte", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_resolved_by_search_path(test_db: PgPool) {
    let setup = r#"
        create schema other;
        create table public.hover_users (id serial primary key, name text not null);
        create table other.hover_users (id serial primary key);
    "#;

    let query = format!(
        "select * from hover_us{}ers",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "table_resolved_by_search_path",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}

const MIXED_CASE_TABLES: &str = r#"
    create table "MyTable" (quoted_id int);
    create table mytable (folded_id int);
    create table "a""b" (escaped_id int);
"#;

// The statements don't parse, so hover finds the tables by name instead of by reference.

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_unquoted_name_is_downcased(test_db: PgPool) {
    let query = format!(
        "select * from MyTa{}ble where",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "table_hover_unquoted_name_is_downcased",
        query,
        Some(MIXED_CASE_TABLES),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_quoted_name_keeps_its_case(test_db: PgPool) {
    let query = format!(
        r#"select * from "MyTa{}ble" where"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "table_hover_quoted_name_keeps_its_case",
        query,
        Some(MIXED_CASE_TABLES),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_table_hover_quoted_name_with_escaped_quote(test_db: PgPool) {
    let query = format!(
        r#"select * from "a""{}b" where"#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "table_hover_quoted_name_with_escaped_quote",
        query,
        Some(MIXED_CASE_TABLES),
        &test_db,
    )
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_not_on_qualifier(test_db: PgPool) {
    let setup = r#"
        create table hover_users (id serial primary key, name text not null);
    "#;

    let query = format!(
        "select {}u.name from hover_users u",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("column_not_on_qualifier", query, Some(setup), &test_db).await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_column_hover_on_last_part_of_quoted_name(test_db: PgPool) {
    let setup = r#"
        create table "Hover.Users" ("Id" int primary key);
    "#;

    let query = format!(
        r#"select "Hover.Users"."I{}d" from "Hover.Users""#,
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor(
        "column_on_last_part_of_quoted_name",
        query,
        Some(setup),
        &test_db,
    )
    .await;
}
