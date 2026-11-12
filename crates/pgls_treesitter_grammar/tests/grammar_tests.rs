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
    let sql2 = "update auth.users as u set email = 'my@mail.com' where u.id = 1;";
    let sql3 = "update auth.users set (email, id) = ('my@mail.com', 1);";

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

#[test]
fn quoted_identifier_with_escaped_quote() {
    let sql = r#"select "a""b".id from "a""b";"#;

    file_snapshot("quoted_identifier_with_escaped_quote", sql);
}

/// Snapshots the tree of a statement that must parse without errors.
fn valid_snapshot(name: &str, sql: &str) {
    let mut parser = tree_sitter::Parser::new();
    let _ = parser.set_language(&pgls_treesitter_grammar::LANGUAGE.into());
    let tree = parser.parse(sql.trim(), None).expect("Unable to parse!");
    assert!(
        !tree.root_node().has_error(),
        "{name} has errors:\n{}",
        printed_tree(sql)
    );

    file_snapshot(name, sql);
}

#[test]
fn postgres_18_returning_old_new() {
    valid_snapshot(
        "postgres_18_returning_with",
        "update t set a = 1 returning with (old as o, new as n) o.a, n.*;",
    );
    valid_snapshot(
        "postgres_18_merge_returning",
        "merge into t using s on t.id = s.id when matched then update set a = s.a returning old.a, new.*;",
    );
}

#[test]
fn postgres_18_generated_columns() {
    valid_snapshot(
        "postgres_18_generated_columns",
        "create table t (a int, b int generated always as (a * 2) virtual, c int generated always as (a + 1), d int generated always as (a) stored);",
    );
}

#[test]
fn postgres_18_constraints() {
    valid_snapshot(
        "postgres_18_enforced",
        "create table t (a int constraint t_a_check check (a > 0) not enforced, b int references u (id) enforced, check (b > 0) not enforced);",
    );
    valid_snapshot(
        "postgres_18_temporal_keys",
        "create table t (id int, valid_at tstzrange, constraint t_pk primary key (id, valid_at without overlaps), unique (id, valid_at without overlaps), constraint t_fk foreign key (id, period valid_at) references u (id, period valid_at));",
    );
    valid_snapshot(
        "postgres_18_not_null_constraint",
        "create table t (a int not null no inherit, b int, constraint t_b_not_null not null b no inherit, not null a);",
    );
    // `period` is an unreserved keyword and stays a valid column name.
    valid_snapshot(
        "postgres_18_period_column",
        "create table t (period int, primary key (period), foreign key (period) references u (period));",
    );
}

#[test]
fn postgres_18_alter_constraint() {
    valid_snapshot(
        "postgres_18_alter_constraint",
        "alter table t alter constraint t_fk not enforced, alter constraint t_check no inherit, alter constraint t_fk deferrable initially deferred;",
    );
    valid_snapshot(
        "postgres_18_add_not_null_not_valid",
        "alter table t add constraint t_a_not_null not null a not valid;",
    );
}

#[test]
fn postgres_18_default_privileges_on_large_objects() {
    valid_snapshot(
        "postgres_18_default_privileges_grant",
        "alter default privileges for role admin in schema s grant select, update on large objects to r with grant option;",
    );
    valid_snapshot(
        "postgres_18_default_privileges_revoke",
        "alter default privileges revoke grant option for all privileges on large objects from public cascade;",
    );
}

#[test]
fn postgres_18_vacuum_and_analyze_only() {
    valid_snapshot("postgres_18_vacuum_only", "vacuum only t, only u (a);");
    valid_snapshot("postgres_18_analyze_only", "analyze only t (a), only u;");
}

#[test]
fn postgres_18_copy_options() {
    valid_snapshot(
        "postgres_18_copy_options",
        "copy t from stdin with (format csv, on_error ignore, reject_limit 10, log_verbosity silent);",
    );
}

#[test]
fn unfinished_option_lists() {
    // While the options are typed, nothing follows them yet.
    valid_snapshot("unfinished_explain_options", "explain (analyze, verbose");
    valid_snapshot("unfinished_analyze_options", "analyze (verbose");
    valid_snapshot(
        "unfinished_copy_options",
        "copy t from stdin with (format csv",
    );
}

#[test]
fn delete_target_alias_and_using() {
    valid_snapshot(
        "delete_alias_and_using",
        "delete from only users as u using posts p join comments c on c.post_id = p.id where p.user_id = u.id returning u.id;",
    );
    // Postgres takes `ONLY` or `*`, not both.
    valid_snapshot(
        "delete_descendants_alias",
        "delete from users * as u where u.id = 1;",
    );
    valid_snapshot(
        "delete_where_current_of",
        "delete from users u where current of users_cursor;",
    );
}

#[test]
fn vacuum_options() {
    valid_snapshot(
        "vacuum_options",
        "vacuum (full, freeze false, index_cleanup auto, parallel 4, buffer_usage_limit '256kB') only users *, posts (id);",
    );
    valid_snapshot(
        "vacuum_legacy_options",
        "vacuum full freeze verbose analyze users (id);",
    );
    valid_snapshot("unfinished_vacuum_options", "vacuum (verbose");
}

#[test]
fn create_table_constraints_and_inherits() {
    valid_snapshot(
        "create_table_column_constraints",
        "create table t (id int generated by default as identity primary key, name text collate \"C\" not null default 'x', team_id int constraint t_team_fk references teams (id) on delete set null on update cascade);",
    );
    valid_snapshot(
        "create_table_table_constraints",
        "create table t (a int, b int, constraint t_pk primary key (a), unique nulls not distinct (b), foreign key (b) references u (id) on delete no action deferrable, check (a > b));",
    );
    valid_snapshot(
        "create_table_inherits",
        "create table t (a int) inherits (p, q);",
    );
}

#[test]
fn unfinished_create_and_alter_table() {
    // While a constraint is typed, nothing follows it yet.
    valid_snapshot(
        "unfinished_column_constraint",
        "create table t (id int references teams (id) on",
    );
    valid_snapshot(
        "unfinished_table_constraint",
        "create table t (id int, constraint t_fk foreign",
    );
    valid_snapshot(
        "unfinished_alter_column",
        "alter table t alter column a set data",
    );
}

#[test]
fn privilege_names_as_role_names() {
    // Without `ON`, Postgres grants and revokes roles, and reads privilege names as role names.
    valid_snapshot("grant_role_named_usage", "grant usage, update to bob;");
    valid_snapshot("revoke_role_named_usage", "revoke usage from bob;");
    valid_snapshot(
        "grant_usage_privilege",
        "grant usage, update on my_sequence to bob;",
    );
}

#[test]
fn default_privileges_options_in_any_order() {
    valid_snapshot(
        "default_privileges_schema_before_role",
        "alter default privileges in schema public for role admin grant select on tables to bob;",
    );
}
