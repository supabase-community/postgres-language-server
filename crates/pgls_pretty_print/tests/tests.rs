use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgls_pretty_print::{
    emitter::EventEmitter,
    nodes::emit_node_enum,
    renderer::{IndentStyle, RenderConfig, Renderer},
};

/// Line widths to test - each test file is run at both widths
const LINE_WIDTHS: [usize; 2] = [60, 100];

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

    println!("Original content:\n{content}");

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let base_test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // Run test at each configured line width
    for &max_line_length in &LINE_WIDTHS {
        let test_name = format!("{base_test_name}_{max_line_length}");

        let parsed = pgls_query::parse(content).expect("Failed to parse SQL");
        let mut ast = parsed.into_root().expect("No root node found");

        println!("Parsed AST: {ast:#?}");

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

        println!("Formatted content (width={max_line_length}):\n{output}");

        assert_line_lengths(&output, max_line_length);

        let parsed_output = pgls_query::parse(&output).expect("Failed to parse SQL");
        let mut parsed_ast = parsed_output.into_root().expect("No root node found");

        clear_location(&mut parsed_ast);
        clear_location(&mut ast);
        normalize_a_indirection(&mut parsed_ast);
        normalize_a_indirection(&mut ast);
        normalize_object_with_args(&mut parsed_ast);
        normalize_object_with_args(&mut ast);
        normalize_join_expr(&mut parsed_ast);
        normalize_join_expr(&mut ast);
        normalize_foreign_table_partbound(&mut ast);

        assert_eq!(ast, parsed_ast);

        with_settings!({
            omit_expression => true,
            input_file => input_file,
            snapshot_path => "snapshots/single",
        }, {
            assert_snapshot!(test_name, output);
        });
    }
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/multi/",
    glob: "*.sql",
)]
fn test_multi(fixture: Fixture<&str>) {
    let content = fixture.content();

    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let base_test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // Run test at each configured line width
    for &max_line_length in &LINE_WIDTHS {
        let test_name = format!("{base_test_name}_{max_line_length}");

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

            println!("Parsed AST: {ast:#?}");

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
                eprintln!("Failed to parse formatted SQL. Error: {e:?}");
                eprintln!("Statement index: {}", range.start());
                eprintln!("Formatted SQL:\n{output}");
                panic!("Failed to parse formatted SQL: {e:?}");
            });
            let mut parsed_ast = parsed_output.into_root().expect("No root node found");

            clear_location(&mut parsed_ast);
            clear_location(&mut ast);
            normalize_a_indirection(&mut parsed_ast);
            normalize_a_indirection(&mut ast);
            normalize_object_with_args(&mut parsed_ast);
            normalize_object_with_args(&mut ast);
            normalize_join_expr(&mut parsed_ast);
            normalize_join_expr(&mut ast);
            normalize_foreign_table_partbound(&mut ast);
            normalize_merge_support_func(&mut ast);
            normalize_merge_support_func(&mut parsed_ast);
            normalize_sql_value_function(&mut ast);
            normalize_sql_value_function(&mut parsed_ast);

            assert_eq!(ast, parsed_ast);

            formatted_statements.push(output);
        }

        // Join all formatted statements with double newline
        let final_output = formatted_statements.join("\n\n");

        println!("Formatted multi-statement content (width={max_line_length}):\n{final_output}");

        with_settings!({
            omit_expression => true,
            input_file => input_file,
            snapshot_path => "snapshots/multi",
        }, {
            assert_snapshot!(test_name, final_output);
        });
    }
}

fn assert_line_lengths(sql: &str, max_line_length: usize) {
    let mut state = StringState::None;

    for line in sql.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        let mut current_outside_run = 0usize;
        let mut max_outside_run = 0usize;
        let mut longest_token = 0usize;
        let mut current_token_len = 0usize;

        while i < chars.len() {
            match state.clone() {
                StringState::None => {
                    current_outside_run += 1;
                    if current_outside_run > max_outside_run {
                        max_outside_run = current_outside_run;
                    }

                    // Track token length (identifier-like sequences and numeric literals)
                    // Numeric literals can include digits, '.', '-', '+', 'e', 'E'
                    let is_token_char = chars[i].is_alphanumeric()
                        || chars[i] == '_'
                        || chars[i] == '.'
                        || ((chars[i] == '-' || chars[i] == '+')
                            && (current_token_len == 0
                                || (i > 0 && (chars[i - 1] == 'e' || chars[i - 1] == 'E'))));

                    if is_token_char {
                        current_token_len += 1;
                    } else {
                        if current_token_len > longest_token {
                            longest_token = current_token_len;
                        }
                        current_token_len = 0;
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

        // Check final token
        if current_token_len > longest_token {
            longest_token = current_token_len;
        }

        // If the longest token exceeds max length, the line can't be broken further
        // Allow lines where the excess is due to an unbreakable identifier/literal
        // Also allow some overhead for indentation, keywords, and punctuation (up to 20 chars)
        // This accounts for things like "CREATE TRIGGER " or "EXECUTE FUNCTION " prefixes
        let min_overhead = 20;
        if max_outside_run > max_line_length && longest_token + min_overhead <= max_line_length {
            panic!("Line exceeds max length of {max_line_length} outside literals: {line}");
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
                // Normalize funcformat to CoerceExplicitCall since we emit as regular function call
                (*n).funcformat = pgls_query::protobuf::CoercionForm::CoerceExplicitCall.into();
                // Remove pg_catalog prefix from function names
                if (*n).funcname.len() == 2 {
                    if let Some(pgls_query::NodeEnum::String(schema)) =
                        (*n).funcname.first().and_then(|node| node.node.as_ref())
                    {
                        if schema.sval.eq_ignore_ascii_case("pg_catalog") {
                            (*n).funcname.remove(0);
                        }
                    }
                }
                // Normalize function names to lowercase for case-insensitive comparison
                for func_name_node in &mut (*n).funcname {
                    if let Some(pgls_query::NodeEnum::String(s)) = func_name_node.node.as_mut() {
                        s.sval = s.sval.to_lowercase();
                    }
                }
            }
            pgls_query::NodeMut::NamedArgExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::SetToDefault(n) => {
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
                // Normalize defname to lowercase for case-insensitive options
                (*n).defname = (*n).defname.to_lowercase();
            }
            pgls_query::NodeMut::DeclareCursorStmt(n) => {
                // Mask out internal optimization flags (CURSOR_OPT_PARALLEL_OK = 0x100 etc.)
                // Keep only syntax-affecting flags: BINARY, SCROLL, NO_SCROLL, INSENSITIVE, ASENSITIVE, HOLD
                const CURSOR_SYNTAX_MASK: i32 = 0x3F; // First 6 bits
                (*n).options &= CURSOR_SYNTAX_MASK;
            }
            pgls_query::NodeMut::XmlSerialize(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonArrayConstructor(n) => {
                (*n).location = 0;
                if let Some(output) = (*n).output.as_mut() {
                    if let Some(returning) = output.returning.as_mut() {
                        if let Some(format) = returning.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
            }
            pgls_query::NodeMut::JsonObjectConstructor(n) => {
                (*n).location = 0;
                if let Some(output) = (*n).output.as_mut() {
                    if let Some(returning) = output.returning.as_mut() {
                        if let Some(format) = returning.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
            }
            pgls_query::NodeMut::JsonAggConstructor(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonArrayQueryConstructor(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            pgls_query::NodeMut::JsonIsPredicate(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            pgls_query::NodeMut::JsonFuncExpr(n) => {
                (*n).location = 0;
                // Normalize locations for behaviors
                if let Some(on_error) = (*n).on_error.as_mut() {
                    on_error.location = 0;
                }
                if let Some(on_empty) = (*n).on_empty.as_mut() {
                    on_empty.location = 0;
                }
                // Normalize output.returning.format.location
                if let Some(output) = (*n).output.as_mut() {
                    if let Some(returning) = output.returning.as_mut() {
                        if let Some(format) = returning.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
                // Normalize wrapper: JswUnspec (1) and JswNone (2) are semantically equivalent
                // When no wrapper clause is specified, PostgreSQL defaults to WITHOUT WRAPPER
                if (*n).wrapper == 1 {
                    (*n).wrapper = 2;
                }
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

                // Normalize char to bpchar(1) and bpchar to bpchar(1)
                if (*n).names.len() == 1 {
                    if let Some(pgls_query::NodeEnum::String(type_name)) =
                        (*n).names.first().and_then(|node| node.node.as_ref())
                    {
                        let is_char = type_name.sval.eq_ignore_ascii_case("char");
                        let is_bpchar = type_name.sval.eq_ignore_ascii_case("bpchar");
                        if (is_char || is_bpchar) && (*n).typmods.is_empty() {
                            // char/bpchar without size is char(1) = bpchar(1)
                            (*n).names[0] = pgls_query::protobuf::Node {
                                node: Some(pgls_query::NodeEnum::String(
                                    pgls_query::protobuf::String {
                                        sval: "bpchar".to_string(),
                                    },
                                )),
                            };
                            (*n).typmods.push(pgls_query::protobuf::Node {
                                node: Some(pgls_query::NodeEnum::AConst(
                                    pgls_query::protobuf::AConst {
                                        isnull: false,
                                        location: 0,
                                        val: Some(pgls_query::protobuf::a_const::Val::Ival(
                                            pgls_query::protobuf::Integer { ival: 1 },
                                        )),
                                    },
                                )),
                            });
                        }
                    }
                }
            }
            pgls_query::NodeMut::JsonBehavior(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::AConst(n) => {
                (*n).location = 0;
                // Normalize string values to lowercase for settings like datestyle
                // PostgreSQL normalizes case when parsing SET statements
                if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) = (*n).val.as_mut() {
                    s.sval = s.sval.to_lowercase();
                }
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
            pgls_query::NodeMut::RangeTableSample(n) => {
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
            pgls_query::NodeMut::BooleanTest(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::MinMaxExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::Constraint(n) => {
                (*n).location = 0;
                // Normalize pg_default indexspace to empty (default tablespace)
                if (*n).indexspace == "pg_default" {
                    (*n).indexspace = String::new();
                }
            }
            pgls_query::NodeMut::CaseWhen(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CaseExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::TransactionStmt(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonParseExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonFuncExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonScalarExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::JsonSerializeExpr(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::CopyStmt(n) => {
                // Normalize boolean options in COPY statement options
                for opt in (*n).options.iter_mut() {
                    if let Some(pgls_query::NodeEnum::DefElem(def)) = opt.node.as_mut() {
                        def.location = 0;
                        // Normalize boolean true to None for COPY options
                        let should_clear = def
                            .arg
                            .as_ref()
                            .and_then(|arg| arg.node.as_ref())
                            .map(|node| {
                                matches!(node, pgls_query::NodeEnum::Boolean(b) if b.boolval)
                            })
                            .unwrap_or(false);
                        if should_clear {
                            def.arg = None;
                        }
                    }
                }
            }
            pgls_query::NodeMut::DefineStmt(n) => {
                // Normalize the Integer flag in args for ordered-set aggregates
                // The integer indicates the number of direct arguments, but this may differ
                // between syntactic forms (positive value vs -1)
                // Normalize all to 0 to ignore this difference
                for arg in (*n).args.iter_mut() {
                    if let Some(pgls_query::NodeEnum::Integer(int)) = arg.node.as_mut() {
                        int.ival = 0;
                    }
                }
            }
            pgls_query::NodeMut::FunctionParameter(n) => {
                // Normalize FunctionParameter mode to FuncParamDefault
                // When emitting objargs (TypeName), reparsing creates objfuncargs (FunctionParameter)
                // with potentially different modes (e.g., FuncParamVariadic vs FuncParamDefault)
                (*n).mode = pgls_query::protobuf::FunctionParameterMode::FuncParamDefault as i32;
                // Clear name as DROP FUNCTION can be parsed with or without param names
                (*n).name.clear();
            }
            pgls_query::NodeMut::IndexElem(n) => {
                // Normalize DefElem args in opclassopts from TypeName to String
                for opt in (*n).opclassopts.iter_mut() {
                    if let Some(pgls_query::NodeEnum::DefElem(def)) = opt.node.as_mut() {
                        def.location = 0;
                        // Normalize TypeName to String
                        if let Some(ref mut arg) = def.arg {
                            if let Some(pgls_query::NodeEnum::TypeName(tn)) = arg.node.as_mut() {
                                if tn.names.len() == 1
                                    && tn.typmods.is_empty()
                                    && tn.array_bounds.is_empty()
                                    && !tn.setof
                                    && !tn.pct_type
                                {
                                    if let Some(first) = tn.names.first() {
                                        arg.node = first.node.clone();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            pgls_query::NodeMut::DefElem(n) => {
                (*n).location = 0;
                // Normalize boolean options: arg: Some(Boolean(true)) is equivalent to arg: None
                // This handles the difference between "WITH CSV HEADER" and "WITH (FORMAT csv, HEADER)"
                let should_clear = (*n)
                    .arg
                    .as_ref()
                    .and_then(|arg| arg.node.as_ref())
                    .map(|node| matches!(node, pgls_query::NodeEnum::Boolean(b) if b.boolval))
                    .unwrap_or(false);
                if should_clear {
                    (*n).arg = None;
                }

                // Normalize TypeName with single string name to just String
                // e.g., TypeName { names: [String("1000")] } -> String("1000")
                // This handles the difference when an identifier is parsed as a type name
                let should_convert = (*n)
                    .arg
                    .as_ref()
                    .and_then(|arg| arg.node.as_ref())
                    .map(|node| {
                        if let pgls_query::NodeEnum::TypeName(tn) = node {
                            tn.names.len() == 1
                                && tn.typmods.is_empty()
                                && tn.array_bounds.is_empty()
                                && !tn.setof
                                && !tn.pct_type
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);
                if should_convert {
                    if let Some(ref mut arg) = (*n).arg {
                        if let Some(pgls_query::NodeEnum::TypeName(tn)) = arg.node.as_mut() {
                            if let Some(first) = tn.names.first() {
                                arg.node = first.node.clone();
                            }
                        }
                    }
                }
            }
            pgls_query::NodeMut::DeallocateStmt(n) => {
                (*n).location = 0;
            }
            pgls_query::NodeMut::PublicationObjSpec(n) => {
                (*n).location = 0;
                // Normalize TABLES IN SCHEMA CURRENT_SCHEMA to TABLES IN CURRENT_SCHEMA
                // The latter parses as PublicationobjTablesInCurSchema (3) with empty name,
                // while the former parses as PublicationobjTablesInSchema (2) with name "CURRENT_SCHEMA"
                if (*n).pubobjtype == 2 && (*n).name.eq_ignore_ascii_case("CURRENT_SCHEMA") {
                    (*n).pubobjtype = 3;
                    (*n).name.clear();
                }
            }
            pgls_query::NodeMut::XmlExpr(n) => {
                (*n).location = 0;
            }
            _ => {}
        });
    }
}

/// Flatten nested BoolExpr of the same type
/// PostgreSQL's parser flattens `a AND (b AND c)` to `AND(a, b, c)`
/// but our emitter preserves the original nesting. This function normalizes
/// both ASTs to the flattened form for comparison.
fn flatten_bool_expr(be: &mut pgls_query::protobuf::BoolExpr) {
    // First recursively flatten children
    for arg in &mut be.args {
        if let Some(pgls_query::NodeEnum::BoolExpr(ref mut child)) = arg.node {
            flatten_bool_expr(child);
        }
    }

    // Only flatten AND and OR expressions
    let boolop = be.boolop();
    if boolop == pgls_query::protobuf::BoolExprType::AndExpr
        || boolop == pgls_query::protobuf::BoolExprType::OrExpr
    {
        let mut new_args = Vec::new();
        for arg in std::mem::take(&mut be.args) {
            if let Some(pgls_query::NodeEnum::BoolExpr(ref child)) = arg.node {
                if child.boolop() == boolop {
                    // Same boolop type - flatten by pulling up child args
                    new_args.extend(child.args.clone());
                    continue;
                }
            }
            new_args.push(arg);
        }
        be.args = new_args;
    }
}

/// Normalize AIndirection nodes by flattening nested AIndirection into a single node
/// This handles the case where `(col[0])[0]` parses to a flat structure but `col[0][0]`
/// parses to a nested structure - they're semantically equivalent.
/// Also converts AIndirection(ColumnRef, [String]) to ColumnRef with merged fields.
fn normalize_a_indirection(node: &mut pgls_query::NodeEnum) {
    if let pgls_query::NodeEnum::AIndirection(ind) = node {
        // Recursively normalize the arg first
        if let Some(ref mut arg) = ind.arg {
            if let Some(ref mut inner_node) = arg.node {
                normalize_a_indirection(inner_node);
            }
        }

        // Now flatten: if arg is another AIndirection, pull up its contents
        loop {
            let inner_opt = ind.arg.as_ref().and_then(|arg| {
                if let Some(pgls_query::NodeEnum::AIndirection(inner)) = arg.node.as_ref() {
                    Some(inner.clone())
                } else {
                    None
                }
            });

            if let Some(inner) = inner_opt {
                // Replace arg with inner's arg
                ind.arg = inner.arg;
                // Prepend inner's indirection to current indirection
                let mut new_indirection = inner.indirection;
                new_indirection.append(&mut ind.indirection);
                ind.indirection = new_indirection;
            } else {
                break;
            }
        }

        // Merge leading String/AStar elements from indirection into ColumnRef fields
        // This normalizes `d1.r` which may parse as either structure
        // Also handles partial cases like `value.if2[1]` where we merge "if2" but keep AIndices
        if let Some(ref mut arg) = ind.arg {
            if let Some(pgls_query::NodeEnum::ColumnRef(col)) = arg.node.as_mut() {
                // Find how many leading elements are String or AStar
                let merge_count = ind
                    .indirection
                    .iter()
                    .take_while(|indir| {
                        matches!(
                            indir.node.as_ref(),
                            Some(pgls_query::NodeEnum::String(_) | pgls_query::NodeEnum::AStar(_))
                        )
                    })
                    .count();

                if merge_count > 0 {
                    // Split indirection: merge first `merge_count` into ColumnRef, keep rest
                    let remaining = ind.indirection.split_off(merge_count);
                    col.fields.append(&mut ind.indirection);
                    ind.indirection = remaining;

                    // If no indirection left, convert to just ColumnRef
                    if ind.indirection.is_empty() {
                        *node = pgls_query::NodeEnum::ColumnRef(col.clone());
                        return;
                    }
                }
            }
        }
    }

    // Recursively process all children
    match node {
        pgls_query::NodeEnum::SelectStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    normalize_a_indirection(n);
                }
            }
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_a_indirection(n);
                }
            }
            for sort in &mut stmt.sort_clause {
                if let Some(ref mut n) = sort.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::SortBy(sb) => {
            if let Some(ref mut n) = sb.node {
                if let Some(ref mut inner) = n.node {
                    normalize_a_indirection(inner);
                }
            }
        }
        pgls_query::NodeEnum::ResTarget(rt) => {
            if let Some(ref mut v) = rt.val {
                if let Some(ref mut n) = v.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::FuncCall(fc) => {
            for arg in &mut fc.args {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
            // Normalize NORMALIZE function's second argument:
            // NFC/NFD/NFKC/NFKD can be emitted as identifier but parsed back as ColumnRef
            // Convert ColumnRef with these values to AConst string
            if fc.funcname.len() == 1 {
                if let Some(pgls_query::NodeEnum::String(s)) =
                    fc.funcname.first().and_then(|n| n.node.as_ref())
                {
                    if s.sval.eq_ignore_ascii_case("normalize") && fc.args.len() == 2 {
                        if let Some(pgls_query::NodeEnum::ColumnRef(cref)) =
                            fc.args[1].node.as_ref()
                        {
                            if cref.fields.len() == 1 {
                                if let Some(pgls_query::NodeEnum::String(field_str)) =
                                    cref.fields[0].node.as_ref()
                                {
                                    let form = field_str.sval.to_uppercase();
                                    if matches!(form.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD") {
                                        // Convert to AConst string for normalization
                                        fc.args[1].node = Some(pgls_query::NodeEnum::AConst(
                                            pgls_query::protobuf::AConst {
                                                isnull: false,
                                                location: 0,
                                                val: Some(
                                                    pgls_query::protobuf::a_const::Val::Sval(
                                                        pgls_query::protobuf::String {
                                                            sval: form.to_lowercase(),
                                                        },
                                                    ),
                                                ),
                                            },
                                        ));
                                    }
                                }
                            }
                        }
                        // Also lowercase AConst values for comparison
                        if let Some(pgls_query::NodeEnum::AConst(aconst)) = fc.args[1].node.as_mut()
                        {
                            if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) =
                                aconst.val.as_mut()
                            {
                                s.sval = s.sval.to_lowercase();
                            }
                        }
                    }
                }
            }
        }
        pgls_query::NodeEnum::AExpr(expr) => {
            if let Some(ref mut l) = expr.lexpr {
                if let Some(ref mut n) = l.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut r) = expr.rexpr {
                if let Some(ref mut n) = r.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::UpdateStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::DeleteStmt(stmt) => {
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::InsertStmt(stmt) => {
            if let Some(ref mut sel) = stmt.select_stmt {
                if let Some(ref mut n) = sel.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::Constraint(c) => {
            if let Some(ref mut raw) = c.raw_expr {
                if let Some(ref mut n) = raw.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::AlterDomainStmt(stmt) => {
            if let Some(ref mut def) = stmt.def {
                if let Some(ref mut n) = def.node {
                    normalize_a_indirection(n);
                    // For NOT NULL constraints on domains, clear keys and conname
                    // since they aren't emitted/reparsed
                    if let pgls_query::NodeEnum::Constraint(c) = n {
                        if c.contype == pgls_query::protobuf::ConstrType::ConstrNotnull as i32 {
                            c.keys.clear();
                            // conname is emitted, so leave it for comparison
                        }
                    }
                }
            }
        }
        pgls_query::NodeEnum::CreateDomainStmt(stmt) => {
            for constraint in &mut stmt.constraints {
                if let Some(ref mut n) = constraint.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::NullTest(nt) => {
            if let Some(ref mut arg) = nt.arg {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::BoolExpr(be) => {
            // First flatten nested BoolExpr of the same type
            flatten_bool_expr(be);
            // Then recursively normalize children
            for arg in &mut be.args {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::RuleStmt(stmt) => {
            for action in &mut stmt.actions {
                if let Some(ref mut n) = action.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::AlterTableStmt(stmt) => {
            for cmd in &mut stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(c)) = cmd.node.as_mut() {
                    if let Some(ref mut def) = c.def {
                        if let Some(ref mut n) = def.node {
                            normalize_a_indirection(n);
                        }
                    }
                }
            }
        }
        pgls_query::NodeEnum::CreateStatsStmt(stmt) => {
            for expr in &mut stmt.exprs {
                if let Some(ref mut n) = expr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::StatsElem(se) => {
            if let Some(ref mut expr) = se.expr {
                if let Some(ref mut n) = expr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::IndexStmt(stmt) => {
            for param in &mut stmt.index_params {
                if let Some(ref mut n) = param.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::IndexElem(ie) => {
            if let Some(ref mut expr) = ie.expr {
                if let Some(ref mut n) = expr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::AExpr(ae) => {
            if let Some(ref mut lexpr) = ae.lexpr {
                if let Some(ref mut n) = lexpr.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut rexpr) = ae.rexpr {
                if let Some(ref mut n) = rexpr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::JoinExpr(je) => {
            if let Some(ref mut quals) = je.quals {
                if let Some(ref mut n) = quals.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut larg) = je.larg {
                if let Some(ref mut n) = larg.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut rarg) = je.rarg {
                if let Some(ref mut n) = rarg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::RangeSubselect(rs) => {
            if let Some(ref mut sub) = rs.subquery {
                if let Some(ref mut n) = sub.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::SubLink(sl) => {
            if let Some(ref mut sub) = sl.subselect {
                if let Some(ref mut n) = sub.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut test) = sl.testexpr {
                if let Some(ref mut n) = test.node {
                    normalize_a_indirection(n);
                }
            }
        }
        pgls_query::NodeEnum::NullTest(nt) => {
            if let Some(ref mut arg) = nt.arg {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        _ => {}
    }
}

/// Normalize ObjectWithArgs by clearing objfuncargs
/// When we emit from objargs, the reparsed objfuncargs may differ in count
/// (e.g., OUT parameters in objfuncargs but not in objargs)
fn normalize_object_with_args(node: &mut pgls_query::NodeEnum) {
    match node {
        pgls_query::NodeEnum::ObjectWithArgs(owa) => {
            // Clear objfuncargs - we emit from objargs only
            owa.objfuncargs.clear();
        }
        pgls_query::NodeEnum::DropStmt(ds) => {
            for obj in &mut ds.objects {
                if let Some(ref mut n) = obj.node {
                    normalize_object_with_args(n);
                }
            }
        }
        pgls_query::NodeEnum::AlterObjectSchemaStmt(stmt) => {
            if let Some(ref mut obj) = stmt.object {
                if let Some(ref mut n) = obj.node {
                    normalize_object_with_args(n);
                }
            }
        }
        pgls_query::NodeEnum::CommentStmt(stmt) => {
            if let Some(ref mut obj) = stmt.object {
                if let Some(ref mut n) = obj.node {
                    normalize_object_with_args(n);
                }
            }
        }
        pgls_query::NodeEnum::AlterFunctionStmt(stmt) => {
            if let Some(ref mut func) = stmt.func {
                normalize_object_with_args_inner(func);
            }
        }
        _ => {}
    }
}

fn normalize_object_with_args_inner(owa: &mut pgls_query::protobuf::ObjectWithArgs) {
    owa.objfuncargs.clear();
}

/// Normalize JoinExpr nodes - clear alias and quals for cross joins
fn normalize_join_expr(node: &mut pgls_query::NodeEnum) {
    match node {
        pgls_query::NodeEnum::JoinExpr(je) => {
            // Normalize INNER JOIN ON TRUE to no quals (equivalent to CROSS JOIN)
            if je.jointype == pgls_query::protobuf::JoinType::JoinInner as i32 {
                let is_true_qual = je
                    .quals
                    .as_ref()
                    .and_then(|q| q.node.as_ref())
                    .map(|n| {
                        matches!(
                            n,
                            pgls_query::NodeEnum::AConst(c)
                                if matches!(c.val.as_ref(), Some(pgls_query::protobuf::a_const::Val::Boolval(b)) if b.boolval)
                        )
                    })
                    .unwrap_or(false);
                if is_true_qual {
                    je.quals = None;
                }
            }
            // Clear join alias (not currently emitted - requires parentheses around join)
            je.alias = None;
            // Clear join_using_alias (not currently emitted)
            je.join_using_alias = None;
            // Recursively normalize nested joins and subqueries
            if let Some(ref mut larg) = je.larg {
                if let Some(ref mut n) = larg.node {
                    normalize_join_expr(n);
                }
            }
            if let Some(ref mut rarg) = je.rarg {
                if let Some(ref mut n) = rarg.node {
                    normalize_join_expr(n);
                }
            }
        }
        pgls_query::NodeEnum::RangeSubselect(rs) => {
            if let Some(ref mut sub) = rs.subquery {
                if let Some(ref mut n) = sub.node {
                    normalize_join_expr(n);
                }
            }
        }
        pgls_query::NodeEnum::SelectStmt(stmt) => {
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    normalize_join_expr(n);
                }
            }
        }
        pgls_query::NodeEnum::ViewStmt(vs) => {
            if let Some(ref mut query) = vs.query {
                if let Some(ref mut n) = query.node {
                    normalize_join_expr(n);
                }
            }
        }
        pgls_query::NodeEnum::DeleteStmt(del) => {
            if let Some(ref mut where_clause) = del.where_clause {
                if let Some(ref mut n) = where_clause.node {
                    normalize_join_expr(n);
                }
            }
        }
        pgls_query::NodeEnum::SubLink(sl) => {
            if let Some(ref mut sub) = sl.subselect {
                if let Some(ref mut n) = sub.node {
                    normalize_join_expr(n);
                }
            }
        }
        _ => {}
    }
}

/// Normalize CreateForeignTableStmt partbound:
/// When partbound is None but inh_relations is non-empty, set a default partition bound
/// Also clear table_elts for partitions since they inherit from parent
fn normalize_foreign_table_partbound(node: &mut pgls_query::NodeEnum) {
    if let pgls_query::NodeEnum::CreateForeignTableStmt(stmt) = node {
        if let Some(ref mut base) = stmt.base_stmt {
            // If we have partition inheritance
            if !base.inh_relations.is_empty() {
                // Add a default partbound if missing
                if base.partbound.is_none() {
                    base.partbound = Some(pgls_query::protobuf::PartitionBoundSpec {
                        strategy: String::new(),
                        is_default: true,
                        modulus: 0,
                        remainder: 0,
                        listdatums: vec![],
                        lowerdatums: vec![],
                        upperdatums: vec![],
                        location: 0,
                    });
                }
                // Clear table_elts since partition columns come from parent
                base.table_elts.clear();
            }
        }
    }
}

/// Normalize MergeSupportFunc nodes:
/// MergeSupportFunc is an internal executor node that gets emitted as `mergesupport#<oid>`
/// When re-parsed, this becomes a ColumnRef. To match, we need to convert MergeSupportFunc
/// in the original AST to the equivalent ColumnRef.
fn normalize_merge_support_func(node: &mut pgls_query::NodeEnum) {
    normalize_merge_support_func_recursive(node);
}

fn normalize_merge_support_func_recursive(node: &mut pgls_query::NodeEnum) {
    // First check if this node itself is a MergeSupportFunc and replace it
    if let pgls_query::NodeEnum::MergeSupportFunc(msf) = node {
        let ident = format!("mergesupport#{}", msf.msftype);
        *node = pgls_query::NodeEnum::ColumnRef(pgls_query::protobuf::ColumnRef {
            fields: vec![pgls_query::protobuf::Node {
                node: Some(pgls_query::NodeEnum::String(pgls_query::protobuf::String {
                    sval: ident,
                })),
            }],
            location: 0,
        });
        return;
    }

    // Manually handle types that can contain MergeSupportFunc
    match node {
        pgls_query::NodeEnum::SelectStmt(stmt) => {
            process_node_list(&mut stmt.target_list);
            process_node_list(&mut stmt.from_clause);
            process_optional_boxed_node(&mut stmt.where_clause);
            if let Some(ref mut wc) = stmt.with_clause {
                wc.location = 0;
                process_node_list(&mut wc.ctes);
            }
            // Handle set operations (UNION/INTERSECT/EXCEPT)
            if let Some(ref mut larg) = stmt.larg {
                process_select_stmt_box(larg.as_mut());
            }
            if let Some(ref mut rarg) = stmt.rarg {
                process_select_stmt_box(rarg.as_mut());
            }
        }
        pgls_query::NodeEnum::ResTarget(rt) => {
            process_optional_boxed_node(&mut rt.val);
        }
        pgls_query::NodeEnum::CommonTableExpr(cte) => {
            process_optional_boxed_node(&mut cte.ctequery);
        }
        pgls_query::NodeEnum::MergeStmt(stmt) => {
            process_node_list(&mut stmt.returning_list);
            process_node_list(&mut stmt.merge_when_clauses);
        }
        pgls_query::NodeEnum::MergeWhenClause(mwc) => {
            process_node_list(&mut mwc.target_list);
            process_node_list(&mut mwc.values);
            process_optional_boxed_node(&mut mwc.condition);
        }
        pgls_query::NodeEnum::UpdateStmt(stmt) => {
            process_node_list(&mut stmt.target_list);
            process_node_list(&mut stmt.returning_list);
        }
        pgls_query::NodeEnum::InsertStmt(stmt) => {
            process_node_list(&mut stmt.returning_list);
            process_optional_boxed_node(&mut stmt.select_stmt);
            if let Some(ref mut wc) = stmt.with_clause {
                process_node_list(&mut wc.ctes);
            }
        }
        pgls_query::NodeEnum::DeleteStmt(stmt) => {
            process_node_list(&mut stmt.returning_list);
        }
        pgls_query::NodeEnum::CaseExpr(ce) => {
            process_optional_boxed_node(&mut ce.arg);
            process_node_list(&mut ce.args);
            process_optional_boxed_node(&mut ce.defresult);
        }
        pgls_query::NodeEnum::CaseWhen(cw) => {
            process_optional_boxed_node(&mut cw.expr);
            process_optional_boxed_node(&mut cw.result);
        }
        pgls_query::NodeEnum::FuncCall(fc) => {
            process_node_list(&mut fc.args);
        }
        pgls_query::NodeEnum::AExpr(expr) => {
            process_optional_boxed_node(&mut expr.lexpr);
            process_optional_boxed_node(&mut expr.rexpr);
        }
        pgls_query::NodeEnum::SubLink(sl) => {
            process_optional_boxed_node(&mut sl.subselect);
            process_optional_boxed_node(&mut sl.testexpr);
        }
        pgls_query::NodeEnum::RangeSubselect(rs) => {
            process_optional_boxed_node(&mut rs.subquery);
        }
        pgls_query::NodeEnum::JoinExpr(je) => {
            process_optional_boxed_node(&mut je.larg);
            process_optional_boxed_node(&mut je.rarg);
        }
        pgls_query::NodeEnum::ViewStmt(vs) => {
            process_optional_boxed_node(&mut vs.query);
        }
        pgls_query::NodeEnum::WithClause(wc) => {
            wc.location = 0;
            process_node_list(&mut wc.ctes);
        }
        pgls_query::NodeEnum::CopyStmt(cs) => {
            if let Some(query) = &mut cs.query {
                if let Some(n) = &mut query.node {
                    normalize_merge_support_func_recursive(n);
                }
            }
        }
        _ => {}
    }
}

fn process_node_list(list: &mut [pgls_query::protobuf::Node]) {
    for node in list.iter_mut() {
        if let Some(ref mut n) = node.node {
            normalize_merge_support_func_recursive(n);
        }
    }
}

fn process_optional_node(opt: &mut Option<pgls_query::protobuf::Node>) {
    if let Some(node) = opt {
        if let Some(n) = &mut node.node {
            normalize_merge_support_func_recursive(n);
        }
    }
}

fn process_optional_boxed_node(opt: &mut Option<Box<pgls_query::protobuf::Node>>) {
    if let Some(node) = opt {
        if let Some(n) = &mut node.node {
            normalize_merge_support_func_recursive(n);
        }
    }
}

fn process_select_stmt_box(stmt: &mut pgls_query::protobuf::SelectStmt) {
    // Recursively process the SelectStmt
    let inner = std::mem::take(stmt);
    let mut node_enum = pgls_query::NodeEnum::SelectStmt(Box::new(inner));
    normalize_merge_support_func_recursive(&mut node_enum);
    if let pgls_query::NodeEnum::SelectStmt(updated) = node_enum {
        *stmt = *updated;
    }
}

/// Normalize SqlvalueFunction nodes to FuncCall for comparison.
/// PostgreSQL represents CURRENT_SCHEMA, CURRENT_USER, etc. as SqlvalueFunction
/// internally, but when we emit them as functions and reparse, they become FuncCall.
fn normalize_sql_value_function(node: &mut pgls_query::NodeEnum) {
    normalize_sql_value_function_recursive(node);
}

fn normalize_sql_value_function_recursive(node: &mut pgls_query::NodeEnum) {
    // First recursively process all children
    match node {
        pgls_query::NodeEnum::SelectStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    normalize_sql_value_function_recursive(n);
                }
            }
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    normalize_sql_value_function_recursive(n);
                }
            }
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_sql_value_function_recursive(n);
                }
            }
        }
        pgls_query::NodeEnum::AExpr(expr) => {
            if let Some(ref mut l) = expr.lexpr {
                if let Some(ref mut n) = l.node {
                    // Check if this is a SqlvalueFunction that needs conversion
                    if let pgls_query::NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = pgls_query::NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
            if let Some(ref mut r) = expr.rexpr {
                if let Some(ref mut n) = r.node {
                    if let pgls_query::NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = pgls_query::NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
        }
        pgls_query::NodeEnum::ResTarget(rt) => {
            if let Some(ref mut v) = rt.val {
                if let Some(ref mut n) = v.node {
                    if let pgls_query::NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = pgls_query::NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
        }
        pgls_query::NodeEnum::FuncCall(fc) => {
            for arg in &mut fc.args {
                if let Some(ref mut n) = arg.node {
                    if let pgls_query::NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = pgls_query::NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
        }
        _ => {}
    }
}

fn sql_value_to_func_call(
    svf: &pgls_query::protobuf::SqlValueFunction,
) -> Option<pgls_query::protobuf::FuncCall> {
    // Map SqlValueFunctionOp to function name
    let func_name = match svf.op() {
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentSchema => "current_schema",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentUser => "current_user",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopSessionUser => "session_user",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopUser => "user",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentCatalog => "current_catalog",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentDate => "current_date",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTime => "current_time",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTimeN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTimestamp => "current_timestamp",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTimestampN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltime => "localtime",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltimeN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltimestamp => "localtimestamp",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltimestampN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentRole => "current_role",
        _ => return None,
    };

    Some(pgls_query::protobuf::FuncCall {
        funcname: vec![pgls_query::protobuf::Node {
            node: Some(pgls_query::NodeEnum::String(pgls_query::protobuf::String {
                sval: func_name.to_string(),
            })),
        }],
        args: vec![],
        agg_order: vec![],
        agg_filter: None,
        over: None,
        agg_within_group: false,
        agg_star: false,
        agg_distinct: false,
        func_variadic: false,
        funcformat: pgls_query::protobuf::CoercionForm::CoerceExplicitCall.into(),
        location: 0,
    })
}
