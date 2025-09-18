use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgt_pretty_print::{
    emitter::{EventEmitter, ToTokens},
    renderer::{IndentStyle, RenderConfig, Renderer},
};

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/",
    glob: "*.sql",
)]
fn test_formatter(fixture: Fixture<&str>) {
    let content = fixture.content();

    println!("Original content: {}", content);

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

    println!("Original AST: {:?}", ast);

    let mut emitter = EventEmitter::new();
    ast.to_tokens(&mut emitter);

    let mut output = String::new();
    let config = RenderConfig {
        max_line_length,
        indent_size: 2,
        indent_style: IndentStyle::Spaces,
    };
    let mut renderer = Renderer::new(&mut output, config);
    renderer.render(emitter.events).expect("Failed to render");

    println!("Formatted output: {}", output);
    let parsed_output = pgt_query::parse(&output).expect("Failed to parse SQL");
    let mut parsed_ast = parsed_output.into_root().expect("No root node found");

    // the location fields are now different in the two ASTs
    clear_location_recursive(&mut parsed_ast);
    clear_location_recursive(&mut ast);

    assert_eq!(ast, parsed_ast);

    with_settings!({
      omit_expression => true,
      input_file => input_file,
    }, {
      assert_snapshot!(test_name, output);
    });
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/",
    glob: "*.sql",
)]
fn validate_test_data(fixture: Fixture<&str>) {
    let content = fixture.content();
    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let _test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    let result = pgt_query::parse(content);

    if let Ok(res) = result.as_ref() {
        assert!(res.root().is_some(), "should have a single root node");
    }

    assert!(result.is_ok(), "should be valid SQL");
}

fn clear_location_recursive(node: &mut pgt_query::NodeEnum) {
    clear_location(node);

    // Recursively clear for each node type that contains other nodes
    match node {
        pgt_query::NodeEnum::SelectStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    clear_location_recursive(n);
                }
            }
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut where_clause) = stmt.where_clause {
                if let Some(ref mut n) = where_clause.node {
                    clear_location_recursive(n);
                }
            }
            for group in &mut stmt.group_clause {
                if let Some(ref mut n) = group.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::ResTarget(target) => {
            if let Some(ref mut val) = target.val {
                if let Some(ref mut n) = val.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::AIndirection(ind) => {
            if let Some(ref mut arg) = ind.arg {
                if let Some(ref mut n) = arg.node {
                    clear_location_recursive(n);
                }
            }
            for indirection in &mut ind.indirection {
                if let Some(ref mut n) = indirection.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::RowExpr(expr) => {
            expr.location = 0;
            expr.row_format = pgt_query::protobuf::CoercionForm::CoerceExplicitCall as i32;
            for arg in &mut expr.args {
                if let Some(ref mut n) = arg.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::GroupingFunc(func) => {
            for arg in &mut func.args {
                if let Some(ref mut n) = arg.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::GroupingSet(set) => {
            for content in &mut set.content {
                if let Some(ref mut n) = content.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::RangeVar(_) => {}
        pgt_query::NodeEnum::ColumnRef(col) => {
            for field in &mut col.fields {
                if let Some(ref mut n) = field.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::JoinExpr(join) => {
            if let Some(ref mut larg) = join.larg {
                if let Some(ref mut n) = larg.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut rarg) = join.rarg {
                if let Some(ref mut n) = rarg.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut quals) = join.quals {
                if let Some(ref mut n) = quals.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::AExpr(expr) => {
            if let Some(ref mut lexpr) = expr.lexpr {
                if let Some(ref mut n) = lexpr.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut rexpr) = expr.rexpr {
                if let Some(ref mut n) = rexpr.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::MergeStmt(stmt) => {
            if let Some(ref mut relation) = stmt.relation {
                clear_location_recursive(&mut pgt_query::NodeEnum::RangeVar(relation.clone()));
            }
            if let Some(ref mut source_relation) = stmt.source_relation {
                if let Some(ref mut n) = source_relation.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut join_condition) = stmt.join_condition {
                if let Some(ref mut n) = join_condition.node {
                    clear_location_recursive(n);
                }
            }
            for when_clause in &mut stmt.merge_when_clauses {
                if let Some(ref mut n) = when_clause.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::MergeWhenClause(clause) => {
            for target in &mut clause.target_list {
                if let Some(ref mut n) = target.node {
                    clear_location_recursive(n);
                }
            }
            for value in &mut clause.values {
                if let Some(ref mut n) = value.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::CreateStmt(stmt) => {
            if let Some(ref mut relation) = stmt.relation {
                clear_location_recursive(&mut pgt_query::NodeEnum::RangeVar(relation.clone()));
            }
            for elt in &mut stmt.table_elts {
                if let Some(ref mut n) = elt.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut partspec) = stmt.partspec {
                clear_location_recursive(&mut pgt_query::NodeEnum::PartitionSpec(partspec.clone()));
            }
        }
        pgt_query::NodeEnum::PartitionSpec(spec) => {
            spec.location = 0;
            for param in &mut spec.part_params {
                if let Some(ref mut n) = param.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::PartitionElem(elem) => {
            elem.location = 0;
        }
        pgt_query::NodeEnum::ColumnDef(def) => {
            def.location = 0;
            if let Some(ref mut type_name) = def.type_name {
                clear_location_recursive(&mut pgt_query::NodeEnum::TypeName(type_name.clone()));
            }
        }
        pgt_query::NodeEnum::TypeName(name) => {
            name.location = 0;
        }
        pgt_query::NodeEnum::RangeSubselect(subselect) => {
            if let Some(ref mut subquery) = subselect.subquery {
                if let Some(ref mut n) = subquery.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::AConst(const_val) => {
            const_val.location = 0;
        }
        pgt_query::NodeEnum::JsonTable(table) => {
            table.location = 0;
            if let Some(ref mut context_item) = table.context_item {
                if let Some(ref mut raw_expr) = context_item.raw_expr {
                    if let Some(ref mut n) = raw_expr.node {
                        clear_location_recursive(n);
                    }
                }
            }
            if let Some(ref mut pathspec) = table.pathspec {
                pathspec.location = 0;
                if let Some(ref mut string) = pathspec.string {
                    if let Some(ref mut n) = string.node {
                        clear_location_recursive(n);
                    }
                }
            }
            for col in &mut table.columns {
                if let Some(ref mut n) = col.node {
                    clear_location_recursive(n);
                }
            }
        }
        pgt_query::NodeEnum::JsonTableColumn(col) => {
            col.location = 0;
            if let Some(ref mut pathspec) = col.pathspec {
                pathspec.location = 0;
                if let Some(ref mut string) = pathspec.string {
                    if let Some(ref mut n) = string.node {
                        clear_location_recursive(n);
                    }
                }
            }
        }
        pgt_query::NodeEnum::TypeCast(cast) => {
            cast.location = 0;
            if let Some(ref mut arg) = cast.arg {
                if let Some(ref mut n) = arg.node {
                    clear_location_recursive(n);
                }
            }
            if let Some(ref mut type_name) = cast.type_name {
                clear_location_recursive(&mut pgt_query::NodeEnum::TypeName(type_name.clone()));
            }
        }
        _ => {}
    }
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
            pgt_query::NodeMut::TypeName(n) => {
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
            pgt_query::NodeMut::AIndirection(_n) => {
                // AIndirection doesn't have a location field
            }
            pgt_query::NodeMut::RowExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::GroupingFunc(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::GroupingSet(n) => {
                (*n).location = 0;
            }
            _ => {}
        });
    }
}
