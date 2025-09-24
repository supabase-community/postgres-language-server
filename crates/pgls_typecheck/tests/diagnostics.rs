use pgls_console::{
    fmt::{Formatter, HTML},
    markup,
};
use pgls_diagnostics::PrintDiagnostic;
use pgls_typecheck::{TypecheckParams, check_sql};
use sqlx::{Executor, PgPool};

async fn test(name: &str, query: &str, setup: Option<&str>, test_db: &PgPool) {
    if let Some(setup) = setup {
        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");
    }

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .expect("Error loading sql language");

    let schema_cache = pgls_schema_cache::SchemaCache::load(test_db)
        .await
        .expect("Failed to load Schema Cache");

    let root = pgls_query::parse(query)
        .unwrap()
        .into_root()
        .expect("Failed to parse query");
    let tree = parser.parse(query, None).unwrap();

    let conn = &test_db;
    let result = check_sql(TypecheckParams {
        conn,
        sql: query,
        ast: &root,
        tree: &tree,
        schema_cache: &schema_cache,
        search_path_patterns: vec![],
        identifiers: vec![],
    })
    .await;

    let mut content = vec![];
    let mut writer = HTML::new(&mut content);

    Formatter::new(&mut writer)
        .write_markup(markup! {
            {PrintDiagnostic::simple(&result.unwrap().unwrap())}
        })
        .unwrap();

    let content = String::from_utf8(content).unwrap();

    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, content);
    });
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn invalid_column(pool: PgPool) {
    test(
        "invalid_column",
        "select id, unknown from contacts;",
        Some(
            r#"
        create table public.contacts (
            id serial primary key,
            name varchar(255) not null,
            is_vegetarian bool default false,
            middle_name varchar(255)
        );
    "#,
        ),
        &pool,
    )
    .await;
}
