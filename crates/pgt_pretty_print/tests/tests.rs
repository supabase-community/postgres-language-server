use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgt_pretty_print::{
    emitter::EventEmitter,
    nodes::emit_node_enum,
    renderer::{IndentStyle, RenderConfig, Renderer},
};

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/single/",
    glob: "*.sql",
)]
fn test_single(fixture: Fixture<&str>) {
    let content = fixture.content();

    println!("Original content:\n{}", content);

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // extract line length from filename (e.g., "simple_select_80" -> 80)
    let max_line_length = test_name
        .split('_')
        .next_back()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80);

    let parsed = pgt_query::parse(content).expect("Failed to parse SQL");
    let mut ast = parsed.into_root().expect("No root node found");

    println!("Parsed AST: {:#?}", ast);

    let mut emitter = EventEmitter::new();
    emit_node_enum(&ast, &mut emitter);

    let mut output = String::new();
    let config = RenderConfig {
        max_line_length,
        indent_size: 2,
        indent_style: IndentStyle::Spaces,
    };
    let mut renderer = Renderer::new(&mut output, config);
    renderer.render(emitter.events).expect("Failed to render");

    println!("Formatted content:\n{}", output);

    for line in output.lines() {
        assert!(
            line.len() <= max_line_length,
            "Line exceeds max length of {}: {}",
            max_line_length,
            line
        );
    }

    let parsed_output = pgt_query::parse(&output).expect("Failed to parse SQL");
    let mut parsed_ast = parsed_output.into_root().expect("No root node found");

    clear_location(&mut parsed_ast);
    clear_location(&mut ast);

    assert_eq!(ast, parsed_ast);

    with_settings!({
      omit_expression => true,
      input_file => input_file,
      snapshot_path => "snapshots/single",
    }, {
      assert_snapshot!(test_name, output);
    });
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/multi/",
    glob: "*.sql",
)]
fn test_multi(fixture: Fixture<&str>) {
    let content = fixture.content();

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // extract line length from filename (e.g., "advisory_lock_60" -> 60)
    let max_line_length = test_name
        .split('_')
        .next_back()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(60);

    // Split the content into statements
    let split_result = pgt_statement_splitter::split(content);
    let mut formatted_statements = Vec::new();

    for range in &split_result.ranges {
        let statement = &content[usize::from(range.start())..usize::from(range.end())];
        let trimmed = statement.trim();

        if trimmed.is_empty() {
            continue;
        }

        let parsed = pgt_query::parse(trimmed).expect("Failed to parse SQL");
        let mut ast = parsed.into_root().expect("No root node found");

        println!("Parsed AST: {:#?}", ast);

        let mut emitter = EventEmitter::new();
        emit_node_enum(&ast, &mut emitter);

        let mut output = String::new();
        let config = RenderConfig {
            max_line_length,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
        };
        let mut renderer = Renderer::new(&mut output, config);
        renderer.render(emitter.events).expect("Failed to render");

        // Verify line length
        for line in output.lines() {
            // Allow string literals and JSON content to exceed line length
            let trimmed = line.trim();
            let contains_string =
                trimmed.contains("'") || trimmed.contains("\"") || trimmed.contains("$$");
            let is_json = trimmed.starts_with("'{") || trimmed.starts_with("'[");

            if !contains_string && !is_json {
                assert!(
                    line.len() <= max_line_length,
                    "Line exceeds max length of {}: {}",
                    max_line_length,
                    line
                );
            }
        }

        // Verify AST equality
        let parsed_output = pgt_query::parse(&output).unwrap_or_else(|e| {
            eprintln!("Failed to parse formatted SQL. Error: {:?}", e);
            eprintln!("Statement index: {}", range.start());
            eprintln!("Formatted SQL:\n{}", output);
            panic!("Failed to parse formatted SQL: {:?}", e);
        });
        let mut parsed_ast = parsed_output.into_root().expect("No root node found");

        clear_location(&mut parsed_ast);
        clear_location(&mut ast);

        assert_eq!(ast, parsed_ast);

        formatted_statements.push(output);
    }

    // Join all formatted statements with double newline
    let final_output = formatted_statements.join("\n\n");

    println!("Formatted multi-statement content:\n{}", final_output);

    with_settings!({
        omit_expression => true,
        input_file => input_file,
        snapshot_path => "snapshots/multi",
    }, {
        assert_snapshot!(test_name, final_output);
    });
}

fn clear_location(node: &mut pgt_query::NodeEnum) {
    unsafe {
        node.iter_mut().for_each(|n| match n {
            pgt_query::NodeMut::ColumnRef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::ParamRef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::AExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::JoinExpr(n) => {
                (*n).rtindex = 0;
            }
            pgt_query::NodeMut::TypeCast(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::CollateClause(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::FuncCall(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::AArrayExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::ResTarget(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::SortBy(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::WindowDef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::PartitionSpec(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::PartitionElem(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::SqlvalueFunction(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::ColumnDef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::DefElem(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::XmlSerialize(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::JsonArrayConstructor(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::JsonObjectConstructor(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::JsonAggConstructor(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::JsonTable(n) => {
                (*n).location = 0;
                if let Some(context) = (*n).context_item.as_mut() {
                    if let Some(format) = context.format.as_mut() {
                        format.location = 0;
                    }
                }

                for column in &mut (*n).columns {
                    if let Some(pgt_query::NodeEnum::JsonTableColumn(col)) = column.node.as_mut() {
                        col.location = 0;
                        if let Some(format) = col.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
            }
            pgt_query::NodeMut::JsonTableColumn(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            pgt_query::NodeMut::JsonTablePathSpec(n) => {
                (*n).location = 0;
                (*n).name_location = 0;
            }
            pgt_query::NodeMut::JsonValueExpr(n) => {
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            pgt_query::NodeMut::TypeName(n) => {
                (*n).location = 0;

                if (*n).names.len() == 2 {
                    if let Some(pgt_query::NodeEnum::String(schema)) =
                        (*n).names.first().and_then(|node| node.node.as_ref())
                    {
                        if schema.sval.eq_ignore_ascii_case("pg_catalog") {
                            (*n).names.remove(0);
                        }
                    }
                }
            }
            pgt_query::NodeMut::JsonBehavior(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::AConst(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::RangeVar(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::RoleSpec(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::RangeTableFunc(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::RangeTableFuncCol(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::RowExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::BoolExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::GroupingFunc(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::GroupingSet(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::CommonTableExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::SubLink(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::NullTest(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::Constraint(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::CaseWhen(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::CaseExpr(n) => {
                (*n).location = 0;
            }
            _ => {}
        });
    }
}
