//! Statements whose parts used to run past the line width because they had no break opportunity
//! or broke without an indent.

mod common;

use pgls_pretty_print::{FormatConfig, format_statement};

const WIDTH: usize = 60;

fn format_idempotently(sql: &str) -> String {
    let config = FormatConfig {
        line_width: WIDTH,
        ..Default::default()
    };
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, &config).unwrap().formatted;
    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, &config).unwrap().formatted;
    assert_eq!(first, second);
    first
}

fn assert_fits(sql: &str) -> String {
    let output = format_idempotently(sql);
    for line in output.lines() {
        assert!(
            line.len() <= WIDTH,
            "line of {} columns:\n{output}",
            line.len()
        );
    }
    output
}

#[test]
fn json_aggregates_break_like_a_function_call() {
    assert_eq!(
        assert_fits(
            "select json_arrayagg(some_long_column_name order by another_long_column_name absent on null returning jsonb) from t"
        ),
        "select\n  json_arrayagg(\n    some_long_column_name\n    order by another_long_column_name\n    absent on null\n    returning jsonb\n  )\nfrom\n  t;"
    );
    assert_fits(
        "select json_objectagg(some_long_key_column : some_long_value_column null on null with unique keys returning jsonb) from t",
    );
    // FILTER breaks before the call does.
    let output = assert_fits(
        "select json_arrayagg(foo order by bar) filter (where some_condition_column > 1) from t",
    );
    assert!(
        output.contains("json_arrayagg(foo order by bar absent on null)\n    filter (where"),
        "{output}"
    );
}

#[test]
fn create_subscription_indents_its_clauses() {
    assert_eq!(
        format_idempotently(
            "create subscription my_subscription connection 'host=example.com dbname=postgres' publication pub_one, pub_two with (enabled = false)"
        ),
        "create subscription my_subscription\n  connection 'host=example.com dbname=postgres'\n  publication pub_one, pub_two\n  with (enabled = 'false');"
    );
}

#[test]
fn xml_functions_break_like_a_function_call() {
    assert_fits(
        "select xmlserialize(document some_xml_document_column_name as character varying(1000)) from t",
    );
    assert_fits(
        "select xmlroot(xmlparse(document '<content>some value</content>'), version '1.0', standalone yes) from t",
    );
    assert_fits(
        "select * from t where xmlexists('//town[text() = ''Toronto'']' passing by value some_xml_document_column_name)",
    );
    assert_fits(
        "select xmlelement(name some_element, xmlattributes(some_long_column_name as attribute_name), 'content') from t",
    );
}

#[test]
fn sql_syntax_functions_break_like_a_function_call() {
    assert_eq!(
        assert_fits(
            "select position(some_long_substring_column_name in some_other_long_string_column_name) from t"
        ),
        "select\n  position(\n    some_long_substring_column_name\n    in some_other_long_string_column_name\n  )\nfrom\n  t;"
    );
    assert_fits(
        "select extract(epoch from some_long_timestamp_column_name - another_timestamp_column) from t",
    );
    assert_fits(
        "select overlay(some_long_string_column_name placing some_replacement_column from 2 for 3) from t",
    );
}

#[test]
fn comment_on_breaks_before_is() {
    assert_eq!(
        format_idempotently(
            "comment on column some_schema_name.some_table_name.some_column_name is 'A rather long description'"
        ),
        "comment on column some_schema_name.some_table_name.some_column_name\n  is 'A rather long description';"
    );
}

#[test]
fn event_trigger_tags_fill_indented_lines() {
    assert_fits(
        "create event trigger my_trigger on ddl_command_start when tag in ('CREATE TABLE', 'CREATE INDEX', 'ALTER TABLE', 'DROP TABLE') execute function my_function()",
    );
}

#[test]
fn explain_options_break_like_a_list() {
    assert_fits(
        "explain (analyze true, buffers true, verbose true, format json, costs false) select * from some_table",
    );
}

#[test]
fn is_distinct_from_and_like_wrap_like_a_comparison() {
    assert_eq!(
        assert_fits(
            "select pg_current_xact_id_if_assigned() is not distinct from cast('pg_current_xact_id' as xid8)"
        ),
        "select\n  pg_current_xact_id_if_assigned() is not distinct from\n  cast('pg_current_xact_id' as xid8);"
    );
    assert_fits(
        "select * from t where accounting_lines.accounting_class not similar to '70(1|2|4)%-some-longer-pattern'",
    );
}

#[test]
fn text_after_a_multi_line_literal_has_to_fit_on_its_last_line() {
    let output = format_idempotently(
        "select f('first line\nsecond line', some_long_argument_name, another_long_argument_name, a_third_one) from t",
    );
    assert!(output.contains("\nsecond line',\n"), "{output}");
    assert!(
        common::width_problems(&output, WIDTH, 4).is_empty(),
        "{output}"
    );
}

#[test]
fn the_width_check_counts_literals_but_excuses_unbreakable_lines() {
    // A call with a literal among its arguments could have broken.
    assert_eq!(
        common::width_problems(
            "select f('a fairly long literal', 'and another one', 3);",
            40,
            4
        )
        .len(),
        1
    );
    // A literal that is longer than the line on its own could not.
    assert!(
        common::width_problems(
            "select 'a literal far longer than the line can hold';",
            40,
            4
        )
        .is_empty()
    );
    // Neither could a trailing comment.
    assert!(
        common::width_problems("select 1; -- a comment far longer than the line", 40, 4).is_empty()
    );
}
