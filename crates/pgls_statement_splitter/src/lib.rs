//! Postgres Statement Splitter
//!
//! This crate provides a function to split a SQL source string into individual statements.
pub mod diagnostics;
mod splitter;

use diagnostics::SplitDiagnostic;
use pgls_lexer::Lexer;
use pgls_text_size::TextRange;
use splitter::{Splitter, source};

pub struct SplitResult {
    pub ranges: Vec<TextRange>,
    pub errors: Vec<SplitDiagnostic>,
}

pub fn split(sql: &str) -> SplitResult {
    let lexed = Lexer::new(sql).lex();

    let mut splitter = Splitter::new(&lexed);

    let _ = source(&mut splitter);

    let split_result = splitter.finish();

    let mut errors: Vec<SplitDiagnostic> = lexed.errors().into_iter().map(Into::into).collect();

    errors.extend(
        split_result
            .errors
            .into_iter()
            .map(|err| SplitDiagnostic::from_split_error(err, &lexed)),
    );

    SplitResult {
        ranges: split_result.ranges,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use diagnostics::SplitDiagnostic;
    use ntest::timeout;
    use pgls_lexer::SyntaxKind;
    use pgls_text_size::TextRange;

    use super::*;

    struct Tester {
        input: String,
        result: SplitResult,
    }

    impl From<&str> for Tester {
        fn from(input: &str) -> Self {
            Tester {
                result: split(input),
                input: input.to_string(),
            }
        }
    }

    impl Tester {
        fn assert_single_statement(&self) -> &Self {
            assert_eq!(
                self.result.ranges.len(),
                1,
                "Expected a single statement for input {}, got {}: {:?}",
                self.input,
                self.result.ranges.len(),
                self.result
                    .ranges
                    .iter()
                    .map(|r| &self.input[*r])
                    .collect::<Vec<_>>()
            );
            self
        }

        fn assert_no_errors(&self) -> &Self {
            assert!(
                self.result.errors.is_empty(),
                "Expected no errors, got {}: {:?}",
                self.result.errors.len(),
                self.result.errors
            );
            self
        }

        fn expect_statements(&self, expected: Vec<&str>) -> &Self {
            assert_eq!(
                self.result.ranges.len(),
                expected.len(),
                "Expected {} statements for input\n{}\ngot {}:\n{:?}",
                expected.len(),
                self.input,
                self.result.ranges.len(),
                self.result
                    .ranges
                    .iter()
                    .map(|r| &self.input[*r])
                    .collect::<Vec<_>>()
            );

            for (range, expected) in self.result.ranges.iter().zip(expected.iter()) {
                assert_eq!(*expected, self.input[*range].to_string());
            }

            assert!(
                self.result.ranges.is_sorted_by_key(|r| r.start()),
                "Ranges are not sorted"
            );

            self
        }

        fn expect_errors(&self, expected: Vec<SplitDiagnostic>) -> &Self {
            assert_eq!(
                self.result.errors.len(),
                expected.len(),
                "Expected {} errors, got {}: {:?}",
                expected.len(),
                self.result.errors.len(),
                self.result.errors
            );

            for (err, expected) in self.result.errors.iter().zip(expected.iter()) {
                assert_eq!(expected, err);
            }

            self
        }
    }

    #[test]
    fn begin_commit() {
        Tester::from(
            "BEGIN;
SELECT 1;
COMMIT;",
        )
        .expect_statements(vec!["BEGIN;", "SELECT 1;", "COMMIT;"]);
    }

    #[test]
    fn begin_atomic() {
        Tester::from(
            "CREATE OR REPLACE FUNCTION public.test_fn(some_in TEXT)
RETURNS TEXT
LANGUAGE sql
IMMUTABLE
STRICT
BEGIN ATOMIC
  SELECT $1 || 'foo';
END;",
        )
        .expect_statements(vec![
            "CREATE OR REPLACE FUNCTION public.test_fn(some_in TEXT)
RETURNS TEXT
LANGUAGE sql
IMMUTABLE
STRICT
BEGIN ATOMIC
  SELECT $1 || 'foo';
END;",
        ]);
    }

    #[test]
    fn ts_with_timezone() {
        Tester::from("alter table foo add column bar timestamp with time zone;").expect_statements(
            vec!["alter table foo add column bar timestamp with time zone;"],
        );
    }

    #[test]
    fn test_for_no_key_update() {
        Tester::from(
            "SELECT 1 FROM assessments AS a WHERE a.id = $assessment_id FOR NO KEY UPDATE;",
        )
        .expect_statements(vec![
            "SELECT 1 FROM assessments AS a WHERE a.id = $assessment_id FOR NO KEY UPDATE;",
        ]);
    }

    #[test]
    fn test_crash_eof() {
        Tester::from("CREATE INDEX \"idx_analytics_read_ratio\" ON \"public\".\"message\" USING \"btree\" (\"inbox_id\", \"timestamp\") INCLUDE (\"status\") WHERE (\"is_inbound\" = false and channel_type not in ('postal'', 'sms'));")
            .expect_statements(vec![
                "CREATE INDEX \"idx_analytics_read_ratio\" ON \"public\".\"message\" USING \"btree\" (\"inbox_id\", \"timestamp\") INCLUDE (\"status\") WHERE (\"is_inbound\" = false and channel_type not in ('postal'', 'sms'));",
            ]);
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
    fn grant() {
        let stmts = vec![
            "GRANT SELECT ON TABLE \"public\".\"my_table\" TO \"my_role\";",
            "GRANT UPDATE ON TABLE \"public\".\"my_table\" TO \"my_role\";",
            "GRANT DELETE ON TABLE \"public\".\"my_table\" TO \"my_role\";",
            "GRANT INSERT ON TABLE \"public\".\"my_table\" TO \"my_role\";",
            "GRANT CREATE ON SCHEMA \"public\" TO \"my_role\";",
            "GRANT ALL PRIVILEGES ON DATABASE \"my_database\" TO \"my_role\";",
            "GRANT USAGE ON SCHEMA \"public\" TO \"my_role\";",
            "GRANT EXECUTE ON FUNCTION \"public\".\"my_function\"() TO \"my_role\";",
            "GRANT REFERENCES ON TABLE \"public\".\"my_table\" TO \"my_role\";",
            "GRANT SELECT, UPDATE ON ALL TABLES IN SCHEMA \"public\" TO \"my_role\";",
            "GRANT SELECT, INSERT ON public.users TO anon WITH GRANT OPION GRANTED BY owner;",
            "GRANT owner, admin to anon WITH ADMIN;",
        ];

        for stmt in stmts {
            Tester::from(stmt).expect_statements(vec![stmt]);
        }
    }

    #[test]
    fn revoke() {
        Tester::from("revoke delete on table \"public\".\"voice_call\" from \"anon\";")
            .expect_statements(vec![
                "revoke delete on table \"public\".\"voice_call\" from \"anon\";",
            ]);

        Tester::from("revoke select on table \"public\".\"voice_call\" from \"anon\";")
            .expect_statements(vec![
                "revoke select on table \"public\".\"voice_call\" from \"anon\";",
            ]);

        Tester::from("revoke update on table \"public\".\"voice_call\" from \"anon\";")
            .expect_statements(vec![
                "revoke update on table \"public\".\"voice_call\" from \"anon\";",
            ]);

        Tester::from("revoke insert on table \"public\".\"voice_call\" from \"anon\";")
            .expect_statements(vec![
                "revoke insert on table \"public\".\"voice_call\" from \"anon\";",
            ]);
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
                format!("Expected {:?}", SyntaxKind::INTO_KW),
                TextRange::new(8.into(), 14.into()),
            )]);
    }

    #[test]
    fn command_between_not_starting() {
        Tester::from("select 1\n      \\com test\nselect 2")
            .expect_statements(vec!["select 1", "select 2"]);
    }

    #[test]
    fn command_between() {
        Tester::from("select 1\n\\com test\nselect 2")
            .expect_statements(vec!["select 1", "select 2"]);
    }

    #[test]
    fn command_standalone() {
        Tester::from("select 1\n\n\\com test\n\nselect 2")
            .expect_statements(vec!["select 1", "select 2"]);
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
    fn trigger_instead_of() {
        Tester::from(
            "CREATE OR REPLACE TRIGGER my_trigger
       INSTEAD OF INSERT ON my_table
       FOR EACH ROW
       EXECUTE FUNCTION my_table_trigger_fn();",
        )
        .expect_statements(vec![
            "CREATE OR REPLACE TRIGGER my_trigger
       INSTEAD OF INSERT ON my_table
       FOR EACH ROW
       EXECUTE FUNCTION my_table_trigger_fn();",
        ]);
    }

    #[test]
    fn with_recursive() {
        Tester::from(
            "
WITH RECURSIVE
  template_questions AS (
    -- non-recursive term that finds the ID of the template question (if any) for question_id
    SELECT
      tq.id,
      tq.qid,
      tq.course_id,
      tq.template_directory
    FROM
      questions AS q
      JOIN questions AS tq ON (
        tq.qid = q.template_directory
        AND tq.course_id = q.course_id
      )
    WHERE
      q.id = $question_id
      AND tq.deleted_at IS NULL
      -- required UNION for a recursive WITH statement
    UNION
    -- recursive term that references template_questions again
    SELECT
      tq.id,
      tq.qid,
      tq.course_id,
      tq.template_directory
    FROM
      template_questions AS q
      JOIN questions AS tq ON (
        tq.qid = q.template_directory
        AND tq.course_id = q.course_id
      )
    WHERE
      tq.deleted_at IS NULL
  )
SELECT
  id
FROM
  template_questions
LIMIT
  100;",
        )
        .assert_single_statement()
        .assert_no_errors();
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
    fn revoke_create() {
        Tester::from("REVOKE CREATE ON SCHEMA public FROM introspector_postgres_user;")
            .expect_statements(vec![
                "REVOKE CREATE ON SCHEMA public FROM introspector_postgres_user;",
            ]);
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

    #[test]
    fn merge_into() {
        Tester::from(
            "MERGE INTO course_permissions AS cp
USING (SELECT 1 AS user_id, 2 AS course_id, 'Owner'::enum_course_role AS course_role) AS data
ON (cp.course_id = data.course_id AND cp.user_id = data.user_id)
WHEN MATCHED THEN UPDATE SET course_role = data.course_role
WHEN NOT MATCHED THEN
INSERT
  (user_id, course_id, course_role)
VALUES
  (data.user_id, data.course_id, data.course_role);",
        )
        .expect_statements(vec![
            "MERGE INTO course_permissions AS cp
USING (SELECT 1 AS user_id, 2 AS course_id, 'Owner'::enum_course_role AS course_role) AS data
ON (cp.course_id = data.course_id AND cp.user_id = data.user_id)
WHEN MATCHED THEN UPDATE SET course_role = data.course_role
WHEN NOT MATCHED THEN
INSERT
  (user_id, course_id, course_role)
VALUES
  (data.user_id, data.course_id, data.course_role);",
        ]);
    }

    #[test]
    fn commas_and_newlines() {
        Tester::from(
            "
        select
            email,


        from
            auth.users;
        ",
        )
        .expect_statements(vec![
            "select\n            email,\n\n\n        from\n            auth.users;",
        ]);
    }

    #[test]
    fn does_not_panic_on_eof_expectation() {
        Tester::from("insert").expect_errors(vec![SplitDiagnostic::new(
            "Expected INTO_KW".to_string(),
            TextRange::new(0.into(), 6.into()),
        )]);
    }

    #[test]
    fn does_not_panic_on_incomplete_statements() {
        // does not panic
        let _ = Tester::from("select case ");
    }
}
