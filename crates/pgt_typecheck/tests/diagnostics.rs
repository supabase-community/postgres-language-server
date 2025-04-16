use pgt_console::{
    fmt::{Formatter, HTML},
    markup,
};
use pgt_diagnostics::PrintDiagnostic;
use pgt_test_utils::test_database::get_new_test_db;
use pgt_typecheck::{TypecheckParams, check_sql};
use sqlx::Executor;

async fn test(name: &str, query: &str, setup: &str) {
    let test_db = get_new_test_db().await;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_sql::language())
        .expect("Error loading sql language");

    let root = pgt_query_ext::parse(query).unwrap();
    let tree = parser.parse(query, None).unwrap();

    let conn = &test_db;
    let result = check_sql(TypecheckParams {
        conn,
        sql: query,
        ast: &root,
        tree: &tree,
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

#[tokio::test]
async fn invalid_column() {
    test(
        "invalid_column",
        "select id, unknown from contacts;",
        r#"
        create table public.contacts (
            id serial primary key,
            name varchar(255) not null,
            is_vegetarian bool default false,
            middle_name varchar(255)
        );
    "#,
    )
    .await;
}
