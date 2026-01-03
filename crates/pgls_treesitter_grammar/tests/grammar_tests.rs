use std::fmt::Write;

use insta::assert_snapshot;

fn printed_tree(sql: &str) -> String {
    let sql = sql.trim();
    let mut result = String::new();
    let mut parser = tree_sitter::Parser::new();
    let _ = parser.set_language(&pgls_treesitter_grammar::LANGUAGE.into());
    let tree = parser.parse(sql, None).expect("Unable to parse!");
    pgls_test_utils::print_ts_tree(&tree.root_node(), sql, &mut result);
    result
}

fn file_snapshot(name: &str, sql: &str) {
    let mut writer = String::new();

    write!(&mut writer, "{sql}").unwrap();

    writeln!(&mut writer).unwrap();
    write!(&mut writer, "-----------------------").unwrap();
    writeln!(&mut writer).unwrap();

    write!(&mut writer, "{}", printed_tree(sql)).unwrap();

    assert_snapshot!(name, writer);
}

#[test]
fn test_1() {
    let sql = "select * from auth.users;";

    file_snapshot("test_1", sql);
}

#[test]
fn test_2() {
    let sql1 = "update auth.users set email = 'my@mail.com';";
    let sql2 = "update auth.users set users.email = 'my@mail.com';";
    let sql3 = "update auth.users set auth.users.email = 'my@mail.com';";

    file_snapshot("test_2_sql1", sql1);
    file_snapshot("test_2_sql2", sql2);
    file_snapshot("test_2_sql3", sql3);
}

#[test]
fn test_3() {
    let sql = r#"
select 
    u.id,
    u.email,
    cs.user_settings,
    cs.client_id
from 
    auth.users u
    join public.client_settings cs
    on u.id = cs.user_id;

"#;

    file_snapshot("test_3", sql);
}

#[test]
fn test_4() {
    let sql = r#"select "auth".REPLACED_TOKEN"#;

    file_snapshot("test_4", sql);
}

#[test]
fn test_5() {
    let sql = r#"
  select * from users u
  where u.active = true and (u.role = 'admin' or u.role = 'moderator');
  "#;

    file_snapshot("test_5", sql);
}

#[test]
fn test_6() {
    let sql = r#"
    select (create_composite_type(a, b)).email, (schema.actual_type).id, client from client_settings;
  "#;

    file_snapshot("test_6", sql);
}
