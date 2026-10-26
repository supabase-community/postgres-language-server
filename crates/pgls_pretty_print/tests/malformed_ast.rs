//! The printer writes whatever AST it gets without panicking. An AST the parser wouldn't build
//! gives output with a different AST, which `format_statement` refuses.

use pgls_pretty_print::{FormatConfig, format_statement_unchecked};
use pgls_query::NodeEnum;
use pgls_query::protobuf::{Integer, Node, String as PgString};

fn parse(sql: &str) -> NodeEnum {
    pgls_query::parse(sql)
        .expect("parse")
        .into_root()
        .expect("root")
}

fn print(ast: &NodeEnum) -> String {
    format_statement_unchecked(ast, &FormatConfig::default()).expect("printed")
}

fn integer(ival: i32) -> Node {
    Node {
        node: Some(NodeEnum::Integer(Integer { ival })),
    }
}

fn string(sval: &str) -> Node {
    Node {
        node: Some(NodeEnum::String(PgString {
            sval: sval.to_string(),
        })),
    }
}

#[test]
fn an_unexpected_node_in_a_list_is_printed_as_itself() {
    let mut ast = parse("CREATE TYPE mood AS ENUM ('happy')");
    let NodeEnum::CreateEnumStmt(stmt) = &mut ast else {
        panic!("a CREATE TYPE ... AS ENUM");
    };
    stmt.vals[0] = integer(1);

    assert!(print(&ast).contains("enum (1)"));
}

#[test]
fn an_unexpected_alter_table_command_is_printed_as_itself() {
    let mut ast = parse("ALTER TABLE t ADD COLUMN a int");
    let NodeEnum::AlterTableStmt(stmt) = &mut ast else {
        panic!("an ALTER TABLE");
    };
    stmt.cmds.push(string("b"));

    assert!(print(&ast).contains("b"));
}

/// The value of the first select item of `ast`.
fn first_select_item(ast: &mut NodeEnum) -> &mut NodeEnum {
    let NodeEnum::SelectStmt(select) = ast else {
        panic!("a SELECT");
    };
    let Some(NodeEnum::ResTarget(target)) = select.target_list[0].node.as_mut() else {
        panic!("a select item");
    };
    target
        .val
        .as_mut()
        .and_then(|value| value.node.as_mut())
        .expect("a value")
}

#[test]
fn a_not_with_two_arguments_is_printed() {
    let mut ast = parse("SELECT NOT a FROM t");
    let NodeEnum::BoolExpr(expr) = first_select_item(&mut ast) else {
        panic!("a NOT");
    };
    expr.args.push(expr.args[0].clone());

    print(&ast);
}

#[test]
fn a_constant_without_a_value_is_printed() {
    let mut ast = parse("SELECT 1");
    let NodeEnum::AConst(constant) = first_select_item(&mut ast) else {
        panic!("a constant");
    };
    constant.val = None;

    print(&ast);
}

#[test]
fn a_window_frame_offset_that_is_missing_is_skipped() {
    let mut ast = parse("SELECT count(*) OVER (ROWS 1 PRECEDING) FROM t");
    let NodeEnum::FuncCall(call) = first_select_item(&mut ast) else {
        panic!("a call");
    };
    call.over.as_mut().expect("a window").start_offset = None;

    assert!(print(&ast).contains("rows preceding"));
}
