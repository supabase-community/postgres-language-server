//! Postgres Statement Splitter
//!
//! This crate provides a function to split a SQL source string into individual statements.
pub mod diagnostics;
mod parser;

use parser::{Parse, Parser, source};
use pgt_lexer::diagnostics::ScanError;

pub fn split(sql: &str) -> Result<Parse, Vec<ScanError>> {
    let tokens = pgt_lexer::lex(sql)?;

    let mut parser = Parser::new(tokens);

    source(&mut parser);

    Ok(parser.finish())
}

#[cfg(test)]
mod tests {
    use diagnostics::SplitDiagnostic;
    use ntest::timeout;
    use pgt_lexer::SyntaxKind;
    use pgt_text_size::TextRange;

    use super::*;

    struct Tester {
        input: String,
        parse: Parse,
    }

    impl From<&str> for Tester {
        fn from(input: &str) -> Self {
            Tester {
                parse: split(input).expect("Failed to split"),
                input: input.to_string(),
            }
        }
    }

    impl Tester {
        fn expect_statements(&self, expected: Vec<&str>) -> &Self {
            assert_eq!(
                self.parse.ranges.len(),
                expected.len(),
                "Expected {} statements, got {}: {:?}",
                expected.len(),
                self.parse.ranges.len(),
                self.parse
                    .ranges
                    .iter()
                    .map(|r| &self.input[*r])
                    .collect::<Vec<_>>()
            );

            for (range, expected) in self.parse.ranges.iter().zip(expected.iter()) {
                assert_eq!(*expected, self.input[*range].to_string());
            }

            assert!(
                self.parse.ranges.is_sorted_by_key(|r| r.start()),
                "Ranges are not sorted"
            );

            self
        }

        fn expect_errors(&self, expected: Vec<SplitDiagnostic>) -> &Self {
            assert_eq!(
                self.parse.errors.len(),
                expected.len(),
                "Expected {} errors, got {}: {:?}",
                expected.len(),
                self.parse.errors.len(),
                self.parse.errors
            );

            for (err, expected) in self.parse.errors.iter().zip(expected.iter()) {
                assert_eq!(expected, err);
            }

            self
        }
    }

    #[test]
    fn failing_lexer() {
        let input = "select 1443ddwwd33djwdkjw13331333333333";
        let res = split(input).unwrap_err();
        assert!(!res.is_empty());
    }

    #[test]
    #[timeout(1000)]
    fn basic() {
        Tester::from("select 1 from contact; select 1;")
            .expect_statements(vec!["select 1 from contact;", "select 1;"]);
    }

    #[test]
    fn no_semicolons() {
        Tester::from("select 1 from contact\nselect 1")
            .expect_statements(vec!["select 1 from contact", "select 1"]);
    }

    #[test]
    fn double_newlines() {
        Tester::from("select 1 from contact\n\nselect 1\n\nselect 3").expect_statements(vec![
            "select 1 from contact",
            "select 1",
            "select 3",
        ]);
    }

    #[test]
    fn single_newlines() {
        Tester::from("select 1\nfrom contact\n\nselect 3")
            .expect_statements(vec!["select 1\nfrom contact", "select 3"]);
    }

    #[test]
    fn alter_column() {
        Tester::from("alter table users alter column email drop not null;")
            .expect_statements(vec!["alter table users alter column email drop not null;"]);
    }

    #[test]
    fn insert_expect_error() {
        Tester::from("\ninsert select 1\n\nselect 3")
            .expect_statements(vec!["insert select 1", "select 3"])
            .expect_errors(vec![SplitDiagnostic::new(
                format!("Expected {:?}", SyntaxKind::Into),
                TextRange::new(8.into(), 14.into()),
            )]);
    }

    #[test]
    fn insert_with_select() {
        Tester::from("\ninsert into tbl (id) select 1\n\nselect 3")
            .expect_statements(vec!["insert into tbl (id) select 1", "select 3"]);
    }

    #[test]
    fn c_style_comments() {
        Tester::from("/* this is a test */\nselect 1").expect_statements(vec!["select 1"]);
    }

    #[test]
    fn with_check() {
        Tester::from("create policy employee_insert on journey_execution for insert to authenticated with check ((select private.organisation_id()) = organisation_id);")
            .expect_statements(vec!["create policy employee_insert on journey_execution for insert to authenticated with check ((select private.organisation_id()) = organisation_id);"]);
    }

    #[test]
    fn nested_parenthesis() {
        Tester::from(
            "create table if not exists journey_node_execution (
  id uuid default gen_random_uuid() not null primary key,

  constraint uq_node_exec unique (journey_execution_id, journey_node_id)
);",
        )
        .expect_statements(vec![
            "create table if not exists journey_node_execution (
  id uuid default gen_random_uuid() not null primary key,

  constraint uq_node_exec unique (journey_execution_id, journey_node_id)
);",
        ]);
    }

    #[test]
    fn with_cte() {
        Tester::from("with test as (select 1 as id) select * from test;")
            .expect_statements(vec!["with test as (select 1 as id) select * from test;"]);
    }

    #[test]
    fn case() {
        Tester::from("select case when select 2 then 1 else 0 end")
            .expect_statements(vec!["select case when select 2 then 1 else 0 end"]);
    }

    #[test]
    fn with_security_invoker() {
        Tester::from(
            "create view api.my_view with (security_invoker) as select id from public.my_table;",
        )
        .expect_statements(vec![
            "create view api.my_view with (security_invoker) as select id from public.my_table;",
        ]);
    }

    #[test]
    fn create_trigger() {
        Tester::from("alter table appointment_status add constraint valid_key check (private.strip_special_chars(key) = key and length(key) > 0 and length(key) < 60);

create trigger default_key before insert on appointment_type for each row when (new.key is null) execute procedure default_key ();

create trigger default_key before insert or update on appointment_status for each row when (new.key is null) execute procedure default_key ();

alter table deal_type add column key text not null;
")
            .expect_statements(vec!["alter table appointment_status add constraint valid_key check (private.strip_special_chars(key) = key and length(key) > 0 and length(key) < 60);",
                "create trigger default_key before insert on appointment_type for each row when (new.key is null) execute procedure default_key ();",
                "create trigger default_key before insert or update on appointment_status for each row when (new.key is null) execute procedure default_key ();",
                "alter table deal_type add column key text not null;",
            ]);
    }

    #[test]
    fn policy() {
        Tester::from("create policy employee_tokenauthed_select on provider_template_approval for select to authenticated, tokenauthed using ( select true );")
            .expect_statements(vec!["create policy employee_tokenauthed_select on provider_template_approval for select to authenticated, tokenauthed using ( select true );"]);
    }

    #[test]
    #[timeout(1000)]
    fn simple_select() {
        Tester::from(
            "
select id, name, test1231234123, unknown from co;

select 14433313331333

alter table test drop column id;

select lower('test');
",
        )
        .expect_statements(vec![
            "select id, name, test1231234123, unknown from co;",
            "select 14433313331333",
            "alter table test drop column id;",
            "select lower('test');",
        ]);
    }

    #[test]
    fn create_rule() {
        Tester::from(
            "create rule log_employee_insert as
on insert to employees
do also insert into employee_log (action, employee_id, log_time)
values ('insert', new.id, now());",
        )
        .expect_statements(vec![
            "create rule log_employee_insert as
on insert to employees
do also insert into employee_log (action, employee_id, log_time)
values ('insert', new.id, now());",
        ]);
    }

    #[test]
    fn insert_into() {
        Tester::from("randomness\ninsert into tbl (id) values (1)\nselect 3").expect_statements(
            vec!["randomness", "insert into tbl (id) values (1)\nselect 3"],
        );
    }

    #[test]
    fn update() {
        Tester::from("more randomness\nupdate tbl set col = '1'\n\nselect 3").expect_statements(
            vec!["more randomness", "update tbl set col = '1'", "select 3"],
        );
    }

    #[test]
    fn delete_from() {
        Tester::from("more randomness\ndelete from test where id = 1\n\nselect 3")
            .expect_statements(vec![
                "more randomness",
                "delete from test where id = 1",
                "select 3",
            ]);
    }

    #[test]
    fn with_ordinality() {
        Tester::from("insert into table (col) select 1 from other t cross join lateral jsonb_array_elements(t.buttons) with ordinality as a(b, nr) where t.buttons is not null;").expect_statements(vec!["insert into table (col) select 1 from other t cross join lateral jsonb_array_elements(t.buttons) with ordinality as a(b, nr) where t.buttons is not null;"]);
    }

    #[test]
    fn unknown() {
        Tester::from("random stuff\n\nmore randomness\n\nselect 3").expect_statements(vec![
            "random stuff",
            "more randomness",
            "select 3",
        ]);
    }

    #[test]
    fn unknown_2() {
        Tester::from("random stuff\nselect 1\n\nselect 3").expect_statements(vec![
            "random stuff",
            "select 1",
            "select 3",
        ]);
    }
}
