use pgt_hover::{OnHoverParams, on_hover};
use pgt_schema_cache::SchemaCache;
use pgt_test_utils::QueryWithCursorPosition;
use pgt_text_size::TextSize;
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
        .set_language(tree_sitter_sql::language())
        .expect("Error loading sql language");

    let tree = parser.parse(&sql, None).unwrap();
    let ast = pgt_query::parse(&sql)
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
async fn test_function_hover_builtin_count(test_db: PgPool) {
    let query = format!(
        "select cou{}nt(*) from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("builtin_count", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
async fn test_function_hover_builtin_now(test_db: PgPool) {
    let query = format!(
        "select n{}ow() from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("builtin_now", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
async fn test_function_hover_builtin_max(test_db: PgPool) {
    let query = format!(
        "select m{}ax(id) from users",
        QueryWithCursorPosition::cursor_marker()
    );

    test_hover_at_cursor("builtin_max", query, None, &test_db).await;
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
