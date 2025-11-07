use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgls_pretty_print::{
    emitter::EventEmitter,
    nodes::emit_node_enum,
    renderer::{IndentStyle, RenderConfig, Renderer},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringState {
    None,
    Single,
    Double,
    Dollar(Vec<char>),
}

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

    let parsed = pgls_query::parse(content).expect("Failed to parse SQL");
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

    assert_line_lengths(&output, max_line_length);

    let parsed_output = pgls_query::parse(&output).expect("Failed to parse SQL");
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
    let split_result = pgls_statement_splitter::split(content);
    let mut formatted_statements = Vec::new();

    for range in &split_result.ranges {
        let statement = &content[usize::from(range.start())..usize::from(range.end())];
        let trimmed = statement.trim();

        if trimmed.is_empty() {
            continue;
        }

        let parsed = pgls_query::parse(trimmed).expect("Failed to parse SQL");
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
        assert_line_lengths(&output, max_line_length);

        // Verify AST equality
        let parsed_output = pgls_query::parse(&output).unwrap_or_else(|e| {
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

fn assert_line_lengths(sql: &str, max_line_length: usize) {
    let mut state = StringState::None;

    for line in sql.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        let mut current_outside_run = 0usize;
        let mut max_outside_run = 0usize;

        while i < chars.len() {
            match state.clone() {
                StringState::None => {
                    current_outside_run += 1;
                    if current_outside_run > max_outside_run {
                        max_outside_run = current_outside_run;
                    }

                    match chars[i] {
                        '\'' => {
                            state = StringState::Single;
                            current_outside_run = 0;
                            i += 1;
                        }
                        '"' => {
                            state = StringState::Double;
                            current_outside_run = 0;
                            i += 1;
                        }
                        '$' => {
                            if let Some((tag, len)) = parse_dollar_tag(&chars[i..]) {
                                state = StringState::Dollar(tag);
                                current_outside_run = 0;
                                i += len;
                            } else {
                                i += 1;
                            }
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }
                StringState::Single => {
                    if chars[i] == '\'' {
                        if i + 1 < chars.len() && chars[i + 1] == '\'' {
                            i += 2;
                        } else {
                            state = StringState::None;
                            current_outside_run = 0;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                StringState::Double => {
                    if chars[i] == '"' {
                        if i + 1 < chars.len() && chars[i + 1] == '"' {
                            i += 2;
                        } else {
                            state = StringState::None;
                            current_outside_run = 0;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                StringState::Dollar(tag) => {
                    if chars[i] == '$' && slice_starts_with(&chars[i..], &tag) {
                        state = StringState::None;
                        current_outside_run = 0;
                        i += tag.len();
                    } else {
                        i += 1;
                    }
                }
            }
        }

        if max_outside_run > max_line_length {
            panic!(
                "Line exceeds max length of {} outside literals: {}",
                max_line_length, line
            );
        }
    }
}

fn parse_dollar_tag(chars: &[char]) -> Option<(Vec<char>, usize)> {
    if chars.is_empty() || chars[0] != '$' {
        return None;
    }

    let mut end = 1usize;
    while end < chars.len() {
        let c = chars[end];
        if c.is_ascii_alphanumeric() || c == '_' {
            end += 1;
        } else {
            break;
        }
    }

    if end < chars.len() && chars[end] == '$' {
        let mut tag = Vec::with_capacity(end + 1);
        tag.extend_from_slice(&chars[..=end]);
        Some((tag, end + 1))
    } else {
        None
    }
}

fn slice_starts_with(haystack: &[char], needle: &[char]) -> bool {
    haystack.len() >= needle.len() && haystack[..needle.len()] == needle[..]
}

fn clear_location(node: &mut pgls_query::NodeEnum) {
    unsafe {
        node.iter_mut().for_each(|n| match n {
            pgls_query::NodeMut::ColumnRef(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::ParamRef(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::AExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JoinExpr(n) => {
                (*n).rtindex = 0;
            }
            pgls_query::NodeMut::TypeCast(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CollateClause(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::FuncCall(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::NamedArgExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::AArrayExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::ResTarget(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::SortBy(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CoalesceExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::WindowDef(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::PartitionSpec(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::PartitionElem(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::PartitionBoundSpec(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::SqlvalueFunction(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::ColumnDef(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::DefElem(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::XmlSerialize(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonArrayConstructor(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonObjectConstructor(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonAggConstructor(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonTable(n) => {
                (*n).location = 0;
                if let Some(context) = (*n).context_item.as_mut() {
                    if let Some(format) = context.format.as_mut() {
                        format.location = 0;
                    }
                }

                for column in &mut (*n).columns {
                    if let Some(pgls_query::NodeEnum::JsonTableColumn(col)) = column.node.as_mut() {
                        col.location = 0;
                        if let Some(format) = col.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
            }
            pgls_query::NodeMut::JsonTableColumn(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            pgls_query::NodeMut::JsonTablePathSpec(n) => {
                (*n).location = 0;
                (*n).name_location = 0;
            }
            pgls_query::NodeMut::JsonValueExpr(n) => {
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            pgls_query::NodeMut::OnConflictClause(n) => {
                (*n).location = 0;
                if let Some(infer) = (*n).infer.as_mut() {
                    infer.location = 0;
                }
            }
            pgls_query::NodeMut::InferClause(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::TypeName(n) => {
                (*n).location = 0;

                if (*n).names.len() == 2 {
                    if let Some(pgls_query::NodeEnum::String(schema)) =
                        (*n).names.first().and_then(|node| node.node.as_ref())
                    {
                        if schema.sval.eq_ignore_ascii_case("pg_catalog") {
                            (*n).names.remove(0);
                        }
                    }
                }
            }
            pgls_query::NodeMut::JsonBehavior(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::AConst(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::RangeVar(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::RoleSpec(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::RangeTableFunc(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::RangeTableFuncCol(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::RowExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::BoolExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::GroupingFunc(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::GroupingSet(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CommonTableExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::SubLink(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::NullTest(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::Constraint(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CaseWhen(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CaseExpr(n) => {
                (*n).location = 0;
            }
            _ => {}
        });
    }
}
